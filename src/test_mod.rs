use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn all_case_folders() -> Vec<&'static str> {
    vec![
        "aocs/",
        "ardupilot/",
        "arinc653_annex/",
        "arrays/",
        "bit_codec/",
        "building_control_gen_mixed/",
        "car/",
        "cpp/",
        "data/",
        "fcs/",
        "file-store/",
        "flight-mgmt/",
        "isr/",
        "latency/",
        "line_follower/",
        "lua/",
        "minepump/",
        "minepump_ba/",
        "mixin/",
        "monitor/",
        "mosart/",
        "nested_feature_groups/",
        "packet-store/",
        "pathfinder_system/",
        "periodicDispatch/",
        "ping_spark/",
        "pingpong/",
        "pingpong_timed/",
        "priority_test/",
        "producer-consumer/",
        "producer_consumer_ba/",
        "producer_filter_consumer_mixed/",
        "producer_filter_consumer_periodic/",
        "producer_filter_consumer_sporadic/",
        "radar/",
        "ravenscar/",
        "rma/",
        "rms/",
        "robotv1/",
        "robotv2/",
        "round_robin/",
        "rpc/",
        "satellite/",
        "stm32discovery_ada/",
        "sunseeker/",
        "test_data_port_periodic_domains/",
        "test_event_data_port_periodic_domains/",
        "test_event_port_periodic_domains/",
        "testdpmon-periodic/",
        "testdpmon/",
        "testepmon/",
        "testevent/",
        "testshare/",
        "testsubprogram/",
        "time_triggered/",
        "toy/",
        "wms/",
    ]
}

pub fn run_all_case_folders() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_bin = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let mut failures: Vec<String> = Vec::new();

    for raw in all_case_folders() {
        let folder = normalize_folder(raw);

        let p1 = manifest_dir.join("generate").join("project").join(&folder);
        let p2 = manifest_dir.join(&folder);
        if !p1.exists() && !p2.exists() {
            eprintln!(
                "warning: case folder '{}' not found at '{}' or '{}', will still try to run it",
                folder,
                p1.display(),
                p2.display()
            );
        }

        let ok = run_single_case(&cargo_bin, &manifest_dir, &folder);

        if ok {
            println!("case '{}' OK", folder);
        } else {
            failures.push(folder);
        }
    }

    if !failures.is_empty() {
        panic!(
            "Some cases failed ({}): {}",
            failures.len(),
            failures.join(", ")
        );
    }
}

fn run_single_case(cargo_bin: &str, workdir: &Path, folder: &str) -> bool {
    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(workdir)
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .arg("--input")
        .arg(folder);

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: failed to spawn cargo for case '{}': {}", folder, e);
            return false;
        }
    };

    if output.status.success() {
        true
    } else {
        eprintln!("case '{}' FAILED (status: {})", folder, output.status);

        let out = String::from_utf8_lossy(&output.stdout);
        let err = String::from_utf8_lossy(&output.stderr);

        if !out.trim().is_empty() {
            eprintln!("--- stdout ---\n{}", out);
        }
        if !err.trim().is_empty() {
            eprintln!("--- stderr ---\n{}", err);
        }

        false
    }
}

fn normalize_folder(raw: &str) -> String {
    raw.trim()
        .trim_end_matches('/')
        .trim_end_matches('\\')
        .to_string()
}
