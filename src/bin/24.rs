use itertools::Itertools;
use nom::character::complete::{char, i64, line_ending, space1};
use nom::combinator::{map, map_res};
use nom::multi::separated_list1;
use nom::sequence::{delimited, separated_pair, tuple};
use nom::IResult;

advent_of_code::solution!(24);

type Vec3 = [f64; 3];
type Vec2 = [f64; 2];
type Mat2 = [[f64; 2]; 2];

fn parse_input(input: &str) -> IResult<&str, Vec<(Vec3, Vec3)>> {
    separated_list1(
        line_ending,
        separated_pair(
            parse_vector,
            delimited(space1, char('@'), space1),
            parse_vector,
        ),
    )(input)
}

fn parse_vector(input: &str) -> IResult<&str, Vec3> {
    map_res(
        separated_list1(tuple((char(','), space1)), map(i64, |i| i as f64)),
        |v| v.try_into(),
    )(input)
}

fn solve_part_one(input: &str, min_pos: f64, max_pos: f64) -> Option<usize> {
    let (_, hailstones) = parse_input(input).unwrap();

    hailstones
        .into_iter()
        .tuple_combinations()
        .filter(|(a, b)| a != b)
        .filter(|(a, b)| {
            let (a_pos, a_vel) = *a;
            let (b_pos, b_vel) = *b;

            // Find position c_pos where the trajectories cross (only in the x and y dimensions)
            // (1) c_pos.xy = a_pos.xy + a_vel.xy * t
            // (2) c_pos.xy = b_pos.xy + b_vel.xy * u

            // Set equations (1) and (2) equal to each other:
            // a_pos.xy + a_vel.xy * t = b_pos.xy + b_vel.xy * u

            // Represent as a matrix multiplication equation:
            // | a_vel.x, -b_vel.x | | t | = | b_pos.x - a_pos.x |
            // | a_vel.y, -b_vel.y | | u |   | b_pos.y - a_pos.y |

            // Solve for t and u:
            // | t | = | a_vel.x, -b_vel.x |^-1 | b_pos.x - a_pos.x |
            // | u |   | a_vel.y, -b_vel.y |    | b_pos.y - a_pos.y |

            let matrix: Mat2 = [[a_vel[0], -b_vel[0]], [a_vel[1], -b_vel[1]]];

            let det = matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0];
            // TODO: Add epsilon?
            if det == 0. {
                return false;
            }

            let inv_det = 1. / det;
            let inv_matrix: Mat2 = [
                [matrix[1][1] * inv_det, -matrix[0][1] * inv_det],
                [-matrix[1][0] * inv_det, matrix[0][0] * inv_det],
            ];

            let diff: Vec2 = [b_pos[0] - a_pos[0], b_pos[1] - a_pos[1]];

            let [t, u]: Vec2 = [
                inv_matrix[0][0] * diff[0] + inv_matrix[0][1] * diff[1],
                inv_matrix[1][0] * diff[0] + inv_matrix[1][1] * diff[1],
            ];

            if t < 0. || u < 0. {
                return false;
            }

            let c_pos: Vec3 = [
                a_pos[0] + a_vel[0] * t,
                a_pos[1] + a_vel[1] * t,
                a_pos[2] + a_vel[2] * t,
            ];

            min_pos <= c_pos[0] && c_pos[0] <= max_pos && min_pos <= c_pos[1] && c_pos[1] <= max_pos
        })
        .count()
        .into()
}

pub fn part_one(input: &str) -> Option<usize> {
    solve_part_one(input, 200_000_000_000_000., 400_000_000_000_000.)
}

fn gaussian_elimination<const N: usize, const M: usize>(mut matrix: [[f64; M]; N]) -> [f64; N] {
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

pub fn part_two(input: &str) -> Option<usize> {
    let (_, hailstones) = parse_input(input).unwrap();

    // Find (pos, vel) such that for every (pos_i, vel_i) in hailstones there exists a t_i such that:
    // pos + vel * t_i = pos_i + vel_i * t_i

    // Rewrite as:
    // pos - pos_i = -t_i * (vel - vel_i))

    // Since t_i is a scalar, the two vectors are parallel:
    // (pos - pos_i) x (vel - vel_i) = 0

    // Rewrite to get 3 equations for the scalar components of the cross product
    // (pos.y - pos_i.y) * (vel.z - vel_i.z) - (pos.z - pos_i.z) * (vel.y - vel_i.y) = 0
    // (pos.z - pos_i.z) * (vel.x - vel_i.x) - (pos.x - pos_i.x) * (vel.z - vel_i.z) = 0
    // (pos.x - pos_i.x) * (vel.y - vel_i.y) - (pos.y - pos_i.y) * (vel.x - vel_i.x) = 0

    // Equate first equation for i = 0 and i = 1:
    // (pos.y - pos_0.y) * (vel.z - vel_0.z) - (pos.z - pos_0.z) * (vel.y - vel_0.y) = (pos.y - pos_1.y) * (vel.z - vel_1.z) - (pos.z - pos_1.z) * (vel.y - vel_1.y)
    //
    // Expand:
    // pos.y * vel.z - pos.y * vel_0.z - pos_0.y * vel.z + pos_0.y * vel_0.z - pos.z * vel.y + pos.z * vel_0.y + pos_0.z * vel.y - pos_0.z * vel_0.y
    // = pos.y * vel.z - pos.y * vel_1.z - pos_1.y * vel.z + pos_1.y * vel_1.z - pos.z * vel.y + pos.z * vel_1.y + pos_1.z * vel.y - pos_1.z * vel_1.y
    //
    // Rewrite to simplified linear equation in terms of pos and vel:
    // pos.y * -(vel_0.z - vel_1.z) + vel.z * -(pos_0.y - pos_1.y) + pos.z * (vel_0.y - vel_1.y) + vel.y * (pos_0.z - pos_1.z)
    // = - pos_0.y * vel_0.z + pos_1.y * vel_1.z - pos_1.z * vel_1.y + pos_0.z * vel_0.y

    // Do the same for all three equations for i set to both (0, 1) and (0, 2), and solve the resulting system of linear
    // equations. Note that we have 6 equations and 6 unknowns, so we can use Gaussian elimination to solve the system.

    let p0 = hailstones[0].0;
    let v0 = hailstones[0].1;
    let p1 = hailstones[1].0;
    let v1 = hailstones[1].1;
    let p2 = hailstones[2].0;
    let v2 = hailstones[2].1;

    // Augmented matrix containing coefficients of: pos.x, pos.y, pos.z, vel.x, vel.y, vel.z, constant
    let matrix = [
        [
            0.,
            -(v0[2] - v1[2]),
            v0[1] - v1[1],
            0.,
            p0[2] - p1[2],
            -(p0[1] - p1[1]),
            -p0[1] * v0[2] + p1[1] * v1[2] - p1[2] * v1[1] + p0[2] * v0[1],
        ],
        [
            v0[2] - v1[2],
            0.,
            -(v0[0] - v1[0]),
            -(p0[2] - p1[2]),
            0.,
            p0[0] - p1[0],
            -p0[2] * v0[0] + p1[2] * v1[0] - p1[0] * v1[2] + p0[0] * v0[2],
        ],
        [
            -(v0[1] - v1[1]),
            v0[0] - v1[0],
            0.,
            p0[1] - p1[1],
            -(p0[0] - p1[0]),
            0.,
            -p0[0] * v0[1] + p1[0] * v1[1] - p1[1] * v1[0] + p0[1] * v0[0],
        ],
        [
            0.,
            -(v0[2] - v2[2]),
            v0[1] - v2[1],
            0.,
            p0[2] - p2[2],
            -(p0[1] - p2[1]),
            -p0[1] * v0[2] + p2[1] * v2[2] - p2[2] * v2[1] + p0[2] * v0[1],
        ],
        [
            v0[2] - v2[2],
            0.,
            -(v0[0] - v2[0]),
            -(p0[2] - p2[2]),
            0.,
            p0[0] - p2[0],
            -p0[2] * v0[0] + p2[2] * v2[0] - p2[0] * v2[2] + p0[0] * v0[2],
        ],
        [
            -(v0[1] - v2[1]),
            v0[0] - v2[0],
            0.,
            p0[1] - p2[1],
            -(p0[0] - p2[0]),
            0.,
            -p0[0] * v0[1] + p2[0] * v2[1] - p2[1] * v2[0] + p0[1] * v0[0],
        ],
    ];

    let result = gaussian_elimination(matrix);
    let x = result[0].round() as usize;
    let y = result[1].round() as usize;
    let z = result[2].round() as usize;

    Some(x + y + z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = solve_part_one(
            &advent_of_code::template::read_file("examples", DAY),
            7.,
            27.,
        );
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(47));
    }
}
