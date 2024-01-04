use itertools::Itertools;
use nom::character::complete::{char, i64, line_ending, space1};
use nom::combinator::{map, map_res};
use nom::multi::separated_list1;
use nom::sequence::{delimited, separated_pair, tuple};
use nom::IResult;
use std::fmt::Write;

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

pub fn part_two(input: &str) -> Option<usize> {
    let (_, hailstones) = parse_input(input).unwrap();

    // Find (pos, vel) such that for every (pos_i, vel_i) in hailstones there exists a t_i such that:
    // pos + vel * t_i = pos_i + vel_i * t_i

    for dc in ['x', 'y', 'z'] {
        println!("(declare-const pos_{} Int)", dc);
        println!("(declare-const vel_{} Int)", dc);
    }

    hailstones
        .into_iter()
        .enumerate()
        .for_each(|(i, (pos, vel))| {
            println!("(declare-const t_{} Int)", i);

            for (di, dc) in ['x', 'y', 'z'].into_iter().enumerate() {
                println!(
                    "(assert (= (+ pos_{} (* vel_{} t_{})) (+ {} (* {} t_{}))))",
                    dc, dc, i, pos[di], vel[di], i
                );
            }
        });

    println!("(declare-const result Int)");
    println!("(assert (= result (+ pos_x pos_y pos_z)))");
    println!("(check-sat)");
    println!("(get-value result)");

    println!();
    println!("Run the above SMT-LIB2 code through an SMT solver (e.g. Z3) to get the result");

    None
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
        assert_eq!(result, None);
    }
}
