use std::collections::{HashSet};

advent_of_code::solution!(10);

type Grid = Vec<Vec<u8>>;
type Pt = (usize, usize);

#[inline]
fn dims(g: &Grid) -> (usize, usize) {
    (g.len(), g.first().map(|r| r.len()).unwrap_or(0))
}

#[inline]
fn inb(r: isize, c: isize, h: usize, w: usize) -> bool {
    r >= 0 && c >= 0 && (r as usize) < h && (c as usize) < w
}

#[inline]
fn neighbours((r, c): Pt, h: usize, w: usize) -> impl Iterator<Item = Pt> {
    const D: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    D.into_iter().filter_map(move |(dr, dc)| {
        let (nr, nc) = (r as isize + dr, c as isize + dc);
        if inb(nr, nc, h, w) {
            Some((nr as usize, nc as usize))
        } else {
            None
        }
    })
}

pub fn parse(input: &str) -> Grid {
    input
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            line.trim()
                .chars()
                .map(|ch| ch.to_digit(10).expect("grid must be digits 0-9") as u8)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Part 1: for each trailhead (height==0), count how many DISTINCT peaks (height==9) are reachable
/// via strictly +1 ascents. Sum over all trailheads.
pub fn part_one(input: &str) -> Option<u64> {
    let g = parse(input);
    let (h, w) = dims(&g);
    if h == 0 || w == 0 {
        return None;
    }

    // Collect all trailheads (height 0) once.
    let mut trailheads = Vec::new();
    for r in 0..h {
        for c in 0..w {
            if g[r][c] == 0 {
                trailheads.push((r, c));
            }
        }
    }

    // For each 0, do a bounded DFS/BFS to gather unique 9 positions.
    let mut total = 0usize;
    let mut stack = Vec::new();

    for &start in &trailheads {
        stack.clear();
        stack.push(start);
        let mut seen: HashSet<Pt> = HashSet::new();
        let mut peaks: HashSet<Pt> = HashSet::new();

        while let Some((r, c)) = stack.pop() {
            let here = g[r][c];
            if here == 9 {
                peaks.insert((r, c));
                continue;
            }
            for (nr, nc) in neighbours((r, c), h, w) {
                if g[nr][nc] == here + 1 && seen.insert((nr, nc)) {
                    stack.push((nr, nc));
                }
            }
        }

        total += peaks.len();
    }

    Some(total.try_into().unwrap())
}

/// Part 2: number of DISTINCT ascending paths from each trailhead (0) to ANY peak (9).
/// We can compute the number of paths from every cell to a 9 with DP:
///   ways[r][c] = sum(ways[nr][nc]) over neighbours with height == g[r][c] + 1
/// Base case: ways[r][c] = 1 if g[r][c] == 9.
pub fn part_two(input: &str) -> Option<u64> {
    let g = parse(input);
    let (h, w) = dims(&g);
    if h == 0 || w == 0 {
        return None;
    }

    // Topologically process cells by height increasing 9..=0 reversed (we want children first).
    // We'll fill ways for height 9 first, then 8, ... 0.
    let mut buckets: [Vec<Pt>; 10] = Default::default();
    for r in 0..h {
        for c in 0..w {
            buckets[g[r][c] as usize].push((r, c));
        }
    }

    let mut ways = vec![vec![0usize; w]; h];

    // Base: peaks contribute exactly 1 path (the path that "ends here").
    for &(r, c) in &buckets[9] {
        ways[r][c] = 1;
    }

    // Fill from 8 down to 0.
    for height in (0..=8).rev() {
        let next_h = (height + 1) as u8;
        for &(r, c) in &buckets[height] {
            let mut sum = 0usize;
            for (nr, nc) in neighbours((r, c), h, w) {
                if g[nr][nc] == next_h {
                    sum = sum.saturating_add(ways[nr][nc]); // safe, though AoC inputs won’t overflow
                }
            }
            ways[r][c] = sum;
        }
    }

    // Sum ways for all trailheads.
    let mut total = 0usize;
    for &(r, c) in &buckets[0] {
        total = total.saturating_add(ways[r][c]);
    }
    Some(total.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tiny synthetic sample showing both counting rules.
    ///
    /// Grid:
    /// 0123
    /// 1234
    /// 8765
    /// 9876
    ///
    /// There’s a single trailhead at (0,0)=0.
    /// Valid +1 ascents lead to a unique peak 2 at multiple spots; Part1 counts DISTINCT peaks,
    /// Part2 counts DISTINCT paths.
    const SAMPLE: &str = "\
0123
1234
8765
9876
";

    #[test]
    fn parse_ok() {
        let g = parse(SAMPLE);
        assert_eq!(g.len(), 4);
        assert_eq!(g[0], vec![0, 1, 2, 3]);
    }

    #[test]
    fn part1_sample() {
        // For this mini sample, reachable peaks from the only 0 is at (3,0) => 1.
        assert_eq!(part_one(SAMPLE), Some(1));
    }

    #[test]
    fn part2_sample() {
        // Multiple distinct paths to those peaks; this synthetic expects 3.
        assert_eq!(part_two(SAMPLE), Some(16));
    }

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(36));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(81));
    }
}
