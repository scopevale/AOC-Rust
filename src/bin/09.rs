advent_of_code::solution!(9);

#[derive(Clone, Debug)]
enum Run {
    File { id: u32, len: usize },
    Free { len: usize },
}

fn parse(line: &str) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut file_id: u32 = 0;
    for (i, ch) in line.trim().chars().enumerate() {
        let len = ch.to_digit(10).unwrap() as usize;
        if len == 0 {
            // still keep zero-length runs out entirely
            continue;
        }
        if i % 2 == 0 {
            runs.push(Run::File { id: file_id, len });
            file_id += 1;
        } else {
            runs.push(Run::Free { len });
        }
    }
    runs
}

fn expand_to_blocks(runs: &[Run]) -> Vec<i64> {
    let mut out = Vec::new();
    for r in runs {
        match *r {
            Run::File { id, len } => out.extend(std::iter::repeat(id as i64).take(len)),
            Run::Free { len } => out.extend(std::iter::repeat(-1_i64).take(len)),
        }
    }
    out
}

fn checksum_blocks(blocks: &[i64]) -> u128 {
    // sum i * id for filled cells (id >= 0)
    let mut acc: u128 = 0;
    for (i, &v) in blocks.iter().enumerate() {
        if v >= 0 {
            acc += (i as u128) * (v as u128);
        }
    }
    acc
}

fn solve_part1(runs: &[Run]) -> u128 {
    let mut disk = expand_to_blocks(runs);
    let mut left = 0usize;
    let mut right = disk.len().saturating_sub(1);

    // advance helpers
    let advance_left = |l: &mut usize, d: &Vec<i64>| {
        while *l < d.len() && d[*l] != -1 {
            *l += 1;
        }
    };
    let retreat_right = |r: &mut usize, d: &Vec<i64>| {
        while *r > 0 && d[*r] == -1 {
            *r = r.saturating_sub(1);
        }
    };

    advance_left(&mut left, &disk);
    retreat_right(&mut right, &disk);

    while left < right {
        // move one block from right to left
        disk[left] = disk[right];
        disk[right] = -1;

        // step both pointers
        advance_left(&mut left, &disk);
        if right == 0 {
            break;
        }
        right -= 1;
        retreat_right(&mut right, &disk);
    }

    checksum_blocks(&disk)
}

// --- Part 2 helpers on runs ---

fn merge_adjacent_frees(runs: &mut Vec<Run>) {
    let mut i = 0;
    while i + 1 < runs.len() {
        match (&runs[i], &runs[i + 1]) {
            (Run::Free { len: a }, Run::Free { len: b }) => {
                let new_len = *a + *b;
                runs.splice(i..=i + 1, [Run::Free { len: new_len }]);
                // stay on same i to check if we can merge further
            }
            _ => i += 1,
        }
    }
}

fn find_file_run(runs: &[Run], id: u32) -> Option<usize> {
    runs.iter()
        .position(|r| matches!(r, Run::File { id: fid, .. } if *fid == id))
}

fn runs_to_checksum(runs: &[Run]) -> u128 {
    let mut idx = 0usize;
    let mut acc: u128 = 0;
    for r in runs {
        match *r {
            Run::File { id, len } => {
                // sum over positions idx .. idx+len-1 of (pos * id)
                // = id * (idx + (idx+1) + ... + (idx+len-1))
                // = id * (len * idx + (len-1)*len/2)
                let len_u = len as u128;
                let id_u = id as u128;
                let seq_sum = len_u * (idx as u128) + (len_u.saturating_sub(1) * len_u) / 2;
                acc += id_u * seq_sum;
                idx += len;
            }
            Run::Free { len } => {
                idx += len;
            }
        }
    }
    acc
}

fn solve_part2(mut runs: Vec<Run>) -> u128 {
    // Find max file id
    let max_id = runs
        .iter()
        .filter_map(|r| {
            if let Run::File { id, .. } = r {
                Some(*id)
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);

    for fid in (0..=max_id).rev() {
        // locate the file run index 'k'
        let k = match find_file_run(&runs, fid) {
            Some(i) => i,
            None => continue,
        };

        // compute file length and its absolute block start index
        let (file_len, file_start_idx) = {
            let mut cursor = 0usize;
            let mut file_len = 0usize;
            let mut start = 0usize;
            for (i, r) in runs.iter().enumerate() {
                match *r {
                    Run::File { id, len } if id == fid => {
                        file_len = len;
                        start = cursor;
                        debug_assert_eq!(i, k);
                        break;
                    }
                    Run::File { len, .. } | Run::Free { len } => {
                        cursor += len;
                    }
                }
            }
            (file_len, start) // fixed: return 'start'
        };

        // scan from the left for the first fitting free run that is left of the file
        let mut cursor = 0usize;
        let mut target_run_idx: Option<(usize, usize)> = None; // (run_index, run_len)
        for (i, r) in runs.iter().enumerate() {
            match *r {
                Run::Free { len } => {
                    if len >= file_len && cursor < file_start_idx {
                        target_run_idx = Some((i, len));
                        break;
                    }
                    cursor += len;
                }
                Run::File { len, .. } => cursor += len,
            }
        }

        // if no suitable target, skip moving this file
        let Some((t_idx, t_len)) = target_run_idx else {
            continue;
        };

        // Build replacement at target:
        // - exact fit: replace Free with [File(fid, file_len)]
        // - larger:    replace Free with [File(fid, file_len), Free(t_len - file_len)]
        let mut new_segment = Vec::with_capacity(2);
        new_segment.push(Run::File {
            id: fid,
            len: file_len,
        });
        if t_len > file_len {
            new_segment.push(Run::Free {
                len: t_len - file_len,
            });
        }

        // Splice in the file at the target free run
        runs.splice(t_idx..=t_idx, new_segment);

        // The source file's run index may have shifted by +1 if:
        //  - the target was to the left of the source (t_idx < k), AND
        //  - we inserted two runs (free was larger -> len increased by 1)
        let delta = if t_len > file_len { 1 } else { 0 };
        let k_now = if t_idx < k { k + delta } else { k };

        // Sanity check: we still have the source file at k_now
        debug_assert!(matches!(&runs[k_now],
            Run::File { id, len } if *id == fid && *len == file_len
        ));

        // Replace the source file with a free run of same length
        runs.splice(k_now..=k_now, [Run::Free { len: file_len }]);

        // Keep run list tidy
        merge_adjacent_frees(&mut runs);
    }

    runs_to_checksum(&runs)
}

pub fn part_one(input: &str) -> Option<u64> {
    let line = input.lines().next().unwrap_or("").trim();

    let runs = parse(line);
    let part1 = solve_part1(&runs);

    Some(part1.try_into().unwrap())
}

pub fn part_two(input: &str) -> Option<u64> {
    let line = input.lines().next().unwrap_or("").trim();

    let runs = parse(line);
    let part2 = solve_part2(runs);

    Some(part2.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(1928));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(2858));
    }

    // Official sample from the puzzle statement
    // "2333133121414131402"
    // Expected: Part1 = 1928, Part2 = 2858
    #[test]
    fn sample() {
        let runs = parse("2333133121414131402");
        assert_eq!(solve_part1(&runs), 1928);
        assert_eq!(solve_part2(runs), 2858);
    }
}
