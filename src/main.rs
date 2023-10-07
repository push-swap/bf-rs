extern crate rayon;

use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Copy)]
enum Operation {
    PA = 0,
    PB = 1,
    SA = 2,
    SB = 3,
    SS = 4,
    RA = 5,
    RB = 6,
    RR = 7,
    RRA = 8,
    RRB = 9,
    RRR = 10,
}

static OPERATIONS_IN_EFFICIENCY_ORDER: [Operation; 11] = [
    Operation::RA,
    Operation::RB,
    Operation::RRA,
    Operation::RRB,
    Operation::SA,
    Operation::SB,
    Operation::RR,
    Operation::RRR,
    Operation::SS,
    Operation::PA,
    Operation::PB,
];

static REVERSE_OPERATIONS: [Operation; 11] = [
    Operation::PB,
    Operation::PA,
    Operation::SA,
    Operation::SB,
    Operation::SS,
    Operation::RRA,
    Operation::RRB,
    Operation::RRR,
    Operation::RA,
    Operation::RB,
    Operation::RR,
];

#[derive(Clone)]
struct State {
    left: Vec<u32>,
    right: Vec<u32>,
    operations: Vec<Operation>,
}

type FilterFn = fn(&State) -> bool;

static IS_MEANINGFUL: [FilterFn; 11] = [
    |state| state.right.len() != 0 && state.right[0] != 0,
    |state| state.left.len() != 0 && state.left[0] != 0,
    |state| state.left.len() >= 2 && state.left[0] != 0 && state.left[1] != 0,
    |state| state.right.len() >= 2 && state.right[0] != 0 && state.right[1] != 0,
    |state| IS_MEANINGFUL[2](state) && IS_MEANINGFUL[3](state),
    |state| state.left.len() >= 2 && state.left[0] != 0,
    |state| state.right.len() >= 2 && state.right[0] != 0,
    |state| IS_MEANINGFUL[5](state) && IS_MEANINGFUL[6](state),
    |state| state.left.len() >= 2 && *state.left.last().unwrap() != 0,
    |state| state.right.len() >= 2 && *state.right.last().unwrap() != 0,
    |state| IS_MEANINGFUL[8](state) && IS_MEANINGFUL[9](state),
];

static APPLY: [fn(&State) -> State; 11] = [
    |state| apply_pa(state),
    |state| apply_pb(state),
    |state| apply_sa(state),
    |state| apply_sb(state),
    |state| apply_ss(state),
    |state| apply_ra(state),
    |state| apply_rb(state),
    |state| apply_rr(state),
    |state| apply_rra(state),
    |state| apply_rrb(state),
    |state| apply_rrr(state),
];

fn apply_pa(state: &State) -> State {
    let mut left = vec![state.right[0]];
    left.extend_from_slice(&state.left);
    let mut right = state.right.clone();
    right.remove(0);
    apply_operation(state, Operation::PA, left, right)
}

fn apply_pb(state: &State) -> State {
    let mut right = vec![state.left[0]];
    right.extend_from_slice(&state.right);
    let mut left = state.left.clone();
    left.remove(0);
    apply_operation(state, Operation::PB, left, right)
}

fn apply_sa(state: &State) -> State {
    let mut left = state.left.clone();
    left.swap(0, 1);
    apply_operation(state, Operation::SA, left, state.right.clone())
}

fn apply_sb(state: &State) -> State {
    let mut right = state.right.clone();
    right.swap(0, 1);
    apply_operation(state, Operation::SB, state.left.clone(), right)
}

fn apply_ss(state: &State) -> State {
    let mut left = state.left.clone();
    left.swap(0, 1);
    let mut right = state.right.clone();
    right.swap(0, 1);
    apply_operation(state, Operation::SS, left, right)
}

fn apply_ra(state: &State) -> State {
    let mut left = state.left.clone();
    let a = left.remove(0);
    left.push(a);
    apply_operation(state, Operation::RA, left, state.right.clone())
}

fn apply_rb(state: &State) -> State {
    let mut right = state.right.clone();
    let b = right.remove(0);
    right.push(b);
    apply_operation(state, Operation::RB, state.left.clone(), right)
}

fn apply_rr(state: &State) -> State {
    let mut left = state.left.clone();
    let a = left.remove(0);
    left.push(a);
    let mut right = state.right.clone();
    let b = right.remove(0);
    right.push(b);
    apply_operation(state, Operation::RR, left, right)
}

fn apply_rra(state: &State) -> State {
    let mut left = vec![*state.left.last().unwrap()];
    left.extend_from_slice(&state.left[..state.left.len() - 1]);
    apply_operation(state, Operation::RRA, left, state.right.clone())
}

fn apply_rrb(state: &State) -> State {
    let mut right = vec![*state.right.last().unwrap()];
    right.extend_from_slice(&state.right[..state.right.len() - 1]);
    apply_operation(state, Operation::RRB, state.left.clone(), right)
}

fn apply_rrr(state: &State) -> State {
    let mut left = vec![*state.left.last().unwrap()];
    left.extend_from_slice(&state.left[..state.left.len() - 1]);
    let mut right = vec![*state.right.last().unwrap()];
    right.extend_from_slice(&state.right[..state.right.len() - 1]);
    apply_operation(state, Operation::RRR, left, right)
}

fn apply_operation(state: &State, op: Operation, left: Vec<u32>, right: Vec<u32>) -> State {
    let mut operations = state.operations.clone();
    operations.push(op);
    State {
        left,
        right,
        operations,
    }
}

fn state_key(state: &State) -> String {
    format!(
        "{}/{}",
        state
            .left
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(","),
        state
            .right
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",")
    )
}

fn get_all_cases(initial_state: State) -> (usize, HashMap<String, State>, Vec<Vec<State>>) {
    let mut map = HashMap::new();
    let mut list = Vec::new();
    map.insert(state_key(&initial_state), initial_state.clone());
    list.push(vec![initial_state]);

    for distance in 0.. {
        if list[distance].is_empty() {
            return (distance - 1, map, list);
        }

        let mut next_distance = Vec::new();
        for current_state in &list[distance] {
            for &op in OPERATIONS_IN_EFFICIENCY_ORDER
                .iter()
                .filter(|&&op| IS_MEANINGFUL[op as usize](current_state))
            {
                let next_state = APPLY[op as usize](current_state);
                let key = state_key(&next_state);
                if !map.contains_key(&key) {
                    map.insert(key.clone(), next_state.clone());
                    next_distance.push(next_state);
                }
            }
        }
        list.push(next_distance);
    }

    (0, map, list)
}

fn get_solution<F: Fn(&State) -> bool>(
    left: Vec<u32>,
    right: Vec<u32>,
    filter: F,
) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    let (_, _, list) = get_all_cases(State {
        left,
        right,
        operations: Vec::new(),
    });

    let mut solution = list
        .iter()
        .flatten()
        .filter(|state| filter(state))
        .map(|state| {
            let reversed_operations: Vec<Operation> = state
                .operations
                .iter()
                .rev()
                .map(|&op| REVERSE_OPERATIONS[op as usize])
                .collect();
            (state.left.clone(), reversed_operations)
        })
        .collect::<Vec<_>>();

    solution.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.len().cmp(&b.1.len())));

    let max_operations = solution.iter().map(|(_, ops)| ops.len()).max().unwrap_or(0);

    (max_operations, solution)
}

fn tst(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        (1..=count).chain(std::iter::once(0)).collect(),
        vec![0],
        |state| state.left.len() == count as usize + 1 && state.left[count as usize] == 0,
    )
}

fn tsb(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![0].into_iter().chain(1..=count).collect(),
        vec![0],
        |state| state.left.len() == count as usize + 1 && state.left[count as usize] == 0,
    )
}

fn txt(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        (1..=count).chain(std::iter::once(0)).collect(),
        vec![],
        |state| state.left.len() == count as usize + 1 && state.left[count as usize] == 0,
    )
}

fn txb(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![0].into_iter().chain(1..=count).collect(),
        vec![],
        |state| state.left.len() == count as usize + 1 && state.left[count as usize] == 0,
    )
}

fn tot(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![0],
        (1..=count).chain(std::iter::once(0)).collect(),
        |state| state.left.len() == count as usize + 1 && state.left[count as usize] == 0,
    )
}

fn tos(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(vec![0], (1..=count).collect(), |state| {
        state.left.len() == count as usize + 1 && state.left[count as usize] == 0
    })
}

fn tob(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![0],
        vec![0].into_iter().chain(1..=count).collect(),
        |state| state.left.len() == count as usize + 1 && state.left[count as usize] == 0,
    )
}

fn sss(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution((1..=count).collect(), vec![0], |state| {
        state.left.len() == count as usize
    })
}

fn sxs(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution((1..=count).collect(), vec![], |state| {
        state.left.len() == count as usize
    })
}

fn sot(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![],
        (1..=count).chain(std::iter::once(0)).collect(),
        |state| state.left.len() == count as usize,
    )
}

fn sos(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(vec![], (1..=count).collect(), |state| {
        state.left.len() == count as usize
    })
}

fn sob(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![],
        vec![0].into_iter().chain(1..=count).collect(),
        |state| state.left.len() == count as usize,
    )
}

fn bst(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        (1..=count).chain(std::iter::once(0)).collect(),
        vec![0],
        |state| state.left.len() == count as usize + 1 && state.left[0] == 0,
    )
}

fn bsb(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![0].into_iter().chain(1..=count).collect(),
        vec![0],
        |state| state.left.len() == count as usize + 1 && state.left[0] == 0,
    )
}

fn bxt(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        (1..=count).chain(std::iter::once(0)).collect(),
        vec![],
        |state| state.left.len() == count as usize + 1 && state.left[0] == 0,
    )
}

fn bxb(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![0].into_iter().chain(1..=count).collect(),
        vec![],
        |state| state.left.len() == count as usize + 1 && state.left[0] == 0,
    )
}

fn bot(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![0],
        (1..=count).chain(std::iter::once(0)).collect(),
        |state| state.left.len() == count as usize + 1 && state.left[0] == 0,
    )
}

fn bos(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(vec![0], (1..=count).collect(), |state| {
        state.left.len() == count as usize + 1 && state.left[0] == 0
    })
}

fn bob(count: u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>) {
    get_solution(
        vec![0],
        vec![0].into_iter().chain(1..=count).collect(),
        |state| state.left.len() == count as usize + 1 && state.left[0] == 0,
    )
}

const OPS_TO_STRING: [&str; 11] = [
    "pa", "pb", "sa", "sb", "ss", "ra", "rb", "rr", "rra", "rrb", "rrr",
];

fn ops_to_string(ops: &Vec<Operation>) -> String {
    return ops
        .iter()
        .map(|op| OPS_TO_STRING[*op as usize])
        .collect::<Vec<_>>()
        .join(" ");
}

fn stack_to_string(stack: &Vec<u32>) -> String {
    return stack
        .iter()
        .filter(|x| **x != 0u32)
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" ");
}

fn do_work(count: u32, name: &str, path: fn(u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>)) {
    let (max_ops, solution) = path(count);
    let directory = "generated"; // Specify the directory name
    if !Path::new(directory).exists() {
        if let Err(err) = std::fs::create_dir(directory) {
            eprintln!("Unable to create directory: {:?}", err);
            return;
        }
    }
    let file_path = format!("{}/{}_{}.txt", directory, name, count);
    let mut file = File::create(file_path.clone()).expect("Unable to create file");
    writeln!(file, "Max Operations: {}", max_ops).expect("Unable to write to file");
    for (left, ops) in &solution {
        writeln!(
            file,
            "Stack: {:?}, Operations: {:?}",
            stack_to_string(left),
            ops_to_string(ops)
        )
        .expect("Unable to write to file");
    }
    println!("Done: {}_{}", name, count);
}

const WORK_ITEMS: [(&str, fn(u32) -> (usize, Vec<(Vec<u32>, Vec<Operation>)>)); 19] = [
    ("tst", tst),
    ("tsb", tsb),
    ("txt", txt),
    ("txb", txb),
    ("tot", tot),
    ("tos", tos),
    ("tob", tob),
    ("sss", sss),
    ("sxs", sxs),
    ("sot", sot),
    ("sos", sos),
    ("sob", sob),
    ("bst", bst),
    ("bsb", bsb),
    ("bxt", bxt),
    ("bxb", bxb),
    ("bot", bot),
    ("bos", bos),
    ("bob", bob),
];

fn main() {
    let infinite_work_items = WORK_ITEMS.iter().cycle();
    infinite_work_items
        .enumerate()
        .par_bridge()
        .for_each(|(count, (name, path))| do_work((count / WORK_ITEMS.len()) as u32, *name, *path))
}
