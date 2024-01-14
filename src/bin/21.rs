use advent_of_code::util::{VecSet, VecTable};

advent_of_code::solution!(21);

type CoordT = u32;
type Coord = advent_of_code::util::coord::Coord<CoordT>;
type CoordIndexer = advent_of_code::util::coord::CoordIndexer<CoordT>;
type Grid = VecTable<Coord, bool, CoordIndexer>;

fn parse_input(input: &str) -> (Grid, Coord) {
    let mut width = None;
    let data = input
        .lines()
        .flat_map(|line| {
            if width.is_none() {
                width = Some(line.len());
            } else {
                debug_assert_eq!(width.unwrap(), line.len());
            }
            line.chars().map(|c| c == '#')
        })
        .collect::<Vec<_>>();
    let width = width.unwrap();
    let height = data.len() / width;
    let indexer = CoordIndexer::new(width as CoordT, height as CoordT);
    (
        Grid::from_vec(data, indexer),
        Coord::new(width as CoordT / 2, height as CoordT / 2),
    )
}

fn count_reached_tiles(grid: &Grid, grid_center: Coord, steps: u32) -> Vec<u32> {
    // Indexer for the tiles reachable within the number of steps
    // TODO: This is still wasteful, since we can actually only reach a diamond-shaped subset of the grid
    let full_grid_indexer = CoordIndexer::new(2 * steps + 1, 2 * steps + 1);
    let full_grid_center = Coord::new(steps, steps);

    let mut visited = VecSet::new(full_grid_indexer);
    let mut frontier = vec![full_grid_center];
    visited.insert(full_grid_center);

    let to_grid_coord = |coord: Coord| -> Coord {
        let x = (coord.x + grid_center.x) as i32 - full_grid_center.x as i32;
        let y = (coord.y + grid_center.y) as i32 - full_grid_center.y as i32;
        Coord::new(
            x.rem_euclid(grid.indexer().width as i32) as CoordT,
            y.rem_euclid(grid.indexer().height as i32) as CoordT,
        )
    };

    let mut odd_reached = 1; // Start at 1 because the center is always reached
    let mut even_reached = 0;

    let mut reached = Vec::with_capacity((steps + 1) as usize);
    reached.push(odd_reached);

    for step in 0..steps {
        let mut new_frontier = Vec::new();

        for coord in frontier {
            let neighbors = [
                Coord::new(coord.x - 1, coord.y),
                Coord::new(coord.x + 1, coord.y),
                Coord::new(coord.x, coord.y - 1),
                Coord::new(coord.x, coord.y + 1),
            ];

            for neighbor in neighbors {
                if !visited.insert(neighbor) {
                    continue;
                }

                let grid_coord = to_grid_coord(neighbor);
                if *grid.get(&grid_coord) {
                    continue;
                }

                new_frontier.push(neighbor);
            }
        }

        frontier = new_frontier;

        if step % 2 == 0 {
            even_reached += frontier.len() as u32;
            reached.push(even_reached);
        } else {
            odd_reached += frontier.len() as u32;
            reached.push(odd_reached);
        }
    }

    reached
}

fn gaussian_elimination<const N: usize, const M: usize>(mut matrix: [[f32; M]; N]) -> [f32; N] {
    // TODO: Integer version of this algorithm

    for i in 0..N {
        // Find pivot for column i
        let mut pivot_row = i;
        for j in i + 1..N {
            if matrix[j][i].abs() > matrix[pivot_row][i].abs() {
                pivot_row = j;
            }
        }

        // Swap rows i and pivot_row
        matrix.swap(i, pivot_row);

        // Eliminate column i for rows i+1..N
        for j in i + 1..N {
            let factor = matrix[j][i] / matrix[i][i];
            for k in i..M {
                matrix[j][k] -= factor * matrix[i][k];
            }
        }
    }

    // Back substitution
    let mut x = [0.; N];
    for i in (0..N).rev() {
        x[i] = matrix[i][N];
        for j in i + 1..N {
            x[i] -= matrix[i][j] * x[j];
        }
        x[i] /= matrix[i][i];
    }

    x
}

fn solve_part_one(input: &str, steps: u32) -> Option<u32> {
    let (grid, start) = parse_input(input);
    count_reached_tiles(&grid, start, steps).last().copied()
}

pub fn part_one(input: &str) -> Option<u32> {
    solve_part_one(input, 64)
}

pub fn part_two(input: &str) -> Option<usize> {
    let (grid, start) = parse_input(input);

    // c_x, where x = (num_steps - 65) / 131

    let num_steps = |x: u32| 65 + 131 * x;
    let reached_tiles = count_reached_tiles(&grid, start, num_steps(3));

    let c_0 = reached_tiles[num_steps(0) as usize];
    let c_1 = reached_tiles[num_steps(1) as usize];
    let c_2 = reached_tiles[num_steps(2) as usize];
    let c_3 = reached_tiles[num_steps(3) as usize];

    // There are two types of diamonds in the input grid (A and B). Each diamond (once filled) can be in one of two
    // states, based on parity of number of steps and which ring it's in. These are labeled a1, a2, b1, b2.

    // Counting the number of each type of diamond after `131 * x + 65` steps, we get the following equations for c_x:
    // c_0 = a1
    // c_1 = a2 + (a1+b1)*4
    // c_2 = a1 + (a2+b2)*4 + (a1+b1)*8
    // c_3 = a2 + (a1+b1)*4 + (a2+b2)*8 + (a1+b1)*12
    //
    // c_4 = a1 + (a2+b2)*4 + (a1+b1)*8 + (a2+b2)*12 + (a1+b1)*16
    // c_5 = a2 + (a1+b1)*4 + (a2+b2)*8 + (a1+b1)*12 + (a2+b2)*16 + (a1+b1)*20
    // c_6 = a1 + (a2+b2)*4 + (a1+b1)*8 + (a2+b2)*12 + (a1+b1)*16 + (a2+b2)*20 + (a1+b1)*24
    // c_7 = a2 + (a1+b1)*4 + (a2+b2)*8 + (a1+b1)*12 + (a2+b2)*16 + (a1+b1)*20 + (a2+b2)*24 + (a1+b1)*28

    // Coefficients of (a1+b1) and (a2+b2) respectively:
    // c_0 -> 0, 0
    // c_1 -> 4, 0
    // c_2 -> 8, 4
    // c_3 -> 16, 8
    // c_4 -> 24, 16
    // c_5 -> 36, 24
    // c_6 -> 48, 36
    // c_7 -> 64, 48
    // ...

    // Formula for the coefficients:
    // c_x (if x is odd) -> (x+1)^2, (x-1)^2 + 2(x-1)
    // c_x (if x is even) -> x^2 + 2x, x^2

    // System of equations (as augmented matrix):
    // a1 b1 a2 b2 | c
    let augmented_matrix = [
        [1., 0., 0., 0., c_0 as f32],
        [4., 4., 1., 0., c_1 as f32],
        [9., 8., 4., 4., c_2 as f32],
        [16., 16., 9., 8., c_3 as f32],
    ];

    // Gaussian elimination:
    let [a1, b1, a2, b2] = gaussian_elimination(augmented_matrix);
    let a1 = a1.round() as usize;
    let b1 = b1.round() as usize;
    let a2 = a2.round() as usize;
    let b2 = b2.round() as usize;

    // We can then use these values to get a formula for c_x:
    let c = |x: usize| {
        let (m, c1, c2) = if x % 2 == 0 {
            (a1, x.pow(2) + 2 * x, x.pow(2))
        } else {
            (a2, (x + 1).pow(2), (x - 1).pow(2) + 2 * (x - 1))
        };
        m + (a1 + b1) * c1 + (a2 + b2) * c2
    };

    let num_steps = 26501365;
    let x = (num_steps - 65) / 131;
    Some(c(x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = solve_part_one(&advent_of_code::template::read_file("examples", DAY), 6);
        assert_eq!(result, Some(16));
    }

    #[test]
    fn test_part_two() {
        // The solution to part two makes assumptions about the real input, and so doesn't work for the example input.
    }
}
