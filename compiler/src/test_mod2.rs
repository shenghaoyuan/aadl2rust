use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 维护所有案例文件夹（与 `cargo run -- --input <folder>` 的 <folder> 一一对应）
/// 注意：列表里带不带末尾 `/` 都可以，下面会做 normalize。
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

/// 测试顶层入口：逐个执行 `cargo run -- --input <folder>`
/// - 每个案例失败会记录，但不会立刻中断（与原来一样）
/// - 最后如果存在失败案例，则整体测试失败（panic）
pub fn run_all_case_folders() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_bin = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let mut failures: Vec<String> = Vec::new();

    for raw in all_case_folders() {
        let folder = normalize_folder(raw);

        // 可选：检查 folder 对应路径是否存在
        // 1) 如果输入 folder 就是工程内某个目录名（直接传给程序解析），可以不检查。
        // 2) 如果它对应 `generate/project/<folder>`，可以开启检查，提前发现目录缺失。
        //
        // 下面给一个“弱检查”：只要这两个候选路径都不存在，就给出 warning，但仍然尝试运行。
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

/// 执行单个案例：`cargo run -- --input <folder>`
/// 返回 true 表示成功（exit code = 0）。
fn run_single_case(cargo_bin: &str, workdir: &Path, folder: &str) -> bool {
    // 为了更稳：避免测试里重复增量编译导致输出太冗长，可加 --quiet（按需）
    // 如果想保留所有输出便于定位失败，把 --quiet 去掉即可。
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
        // 打印 stdout/stderr，便于在 CI 或终端直接定位
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

/// 统一 folder 名称：去掉末尾 `/` 或 `\`
fn normalize_folder(raw: &str) -> String {
    raw.trim()
        .trim_end_matches('/')
        .trim_end_matches('\\')
        .to_string()
}
