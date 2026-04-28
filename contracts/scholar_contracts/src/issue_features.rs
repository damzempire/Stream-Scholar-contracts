// Issue #232: milestone dependency graph helpers (DAG validation, descendant closure).

use soroban_sdk::Vec;

fn popcount64(x: u64) -> u32 {
    let mut n = x;
    let mut c = 0u32;
    while n > 0 {
        c += (n & 1) as u32;
        n >>= 1;
    }
    c
}

/// Returns true if the prerequisite graph contains a directed cycle (invalid grant config).
pub fn milestone_graph_has_cycle(milestone_count: u32, parent_masks: &Vec<u64>) -> bool {
    if milestone_count == 0 || milestone_count > 64 {
        return true;
    }
    let n = milestone_count as usize;
    if parent_masks.len() as u32 != milestone_count {
        return true;
    }
    let mut rem = [0u32; 64];
    for i in 0..n {
        let mask = parent_masks.get(i as u32).unwrap_or(0);
        rem[i] = popcount64(mask);
    }
    let mut queue = [0u32; 64];
    let mut qh = 0usize;
    let mut qt = 0usize;
    for i in 0..n {
        if rem[i] == 0 {
            queue[qt] = i as u32;
            qt += 1;
        }
    }
    let mut processed = 0u32;
    while qh < qt {
        let u = queue[qh];
        qh += 1;
        processed += 1;
        for v in 0..n {
            let mask = parent_masks.get(v as u32).unwrap_or(0);
            if (mask >> u) & 1 == 1 {
                rem[v] -= 1;
                if rem[v] == 0 {
                    queue[qt] = v as u32;
                    qt += 1;
                }
            }
        }
    }
    processed != milestone_count
}

/// Bit-index mask of milestones that transitively depend on `revoked_idx`.
pub fn descendant_mask(milestone_count: u32, parent_masks: &Vec<u64>, revoked_idx: u32) -> u64 {
    if revoked_idx >= milestone_count || milestone_count > 64 {
        return 0;
    }
    let n = milestone_count as usize;
    let mut frozen = 0u64;
    let mut frontier = 1u64 << revoked_idx;
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..n {
            if (frozen >> i) & 1 == 1 {
                continue;
            }
            let mask = parent_masks.get(i as u32).unwrap_or(0);
            if mask & frontier != 0 {
                frozen |= 1u64 << i;
                frontier |= 1u64 << i;
                changed = true;
            }
        }
    }
    frozen & !(1u64 << revoked_idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, Vec};

    #[test]
    fn dag_rejects_two_cycle() {
        let env = Env::default();
        let mut v: Vec<u64> = Vec::new(&env);
        v.push_back(2u64);
        v.push_back(1u64);
        assert!(milestone_graph_has_cycle(2, &v));
    }

    #[test]
    fn dag_accepts_diamond() {
        let env = Env::default();
        let mut v: Vec<u64> = Vec::new(&env);
        v.push_back(0);
        v.push_back(0);
        v.push_back(3u64);
        assert!(!milestone_graph_has_cycle(3, &v));
    }

    #[test]
    fn descendants_include_transitive() {
        let env = Env::default();
        let mut v: Vec<u64> = Vec::new(&env);
        v.push_back(0);
        v.push_back(0);
        v.push_back(3u64);
        let d = descendant_mask(3, &v, 0);
        assert_eq!(d & 1, 0);
        // Milestone 2 has parents {0,1}; revoking 0 freezes 2 (bit index 2), not milestone 1.
        assert_ne!(d & (1u64 << 2), 0);
    }
}
