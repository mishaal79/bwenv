use bwenv::env::parser::{read_env_file, validate_env_file, write_env_file};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

fn generate_large_env_vars(count: usize) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    for i in 0..count {
        env_vars.insert(
            format!("VAR_{:06}", i),
            format!("value_for_variable_number_{}_with_some_longer_content", i),
        );
    }
    env_vars
}

fn create_env_file_content(count: usize) -> String {
    let mut content = String::new();
    content.push_str("# Large environment file for benchmarking\n");
    content.push_str("# Generated automatically for performance testing\n\n");

    for i in 0..count {
        if i % 10 == 0 {
            content.push_str(&format!("# Section {}\n", i / 10));
        }
        content.push_str(&format!(
            "VAR_{:06}=value_for_variable_number_{}_with_some_longer_content\n",
            i, i
        ));
        if i % 20 == 19 {
            content.push('\n');
        }
    }
    content
}

fn bench_read_small_env_file(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("small.env");

    let content = create_env_file_content(10);
    fs::write(&file_path, content).unwrap();

    c.bench_function("read_small_env_file", |b| {
        b.iter(|| {
            black_box(read_env_file(&file_path).unwrap());
        })
    });
}

fn bench_read_medium_env_file(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("medium.env");

    let content = create_env_file_content(100);
    fs::write(&file_path, content).unwrap();

    c.bench_function("read_medium_env_file", |b| {
        b.iter(|| {
            black_box(read_env_file(&file_path).unwrap());
        })
    });
}

fn bench_read_large_env_file(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("large.env");

    let content = create_env_file_content(1000);
    fs::write(&file_path, content).unwrap();

    c.bench_function("read_large_env_file", |b| {
        b.iter(|| {
            black_box(read_env_file(&file_path).unwrap());
        })
    });
}

fn bench_write_small_env_file(c: &mut Criterion) {
    let env_vars = generate_large_env_vars(10);

    c.bench_function("write_small_env_file", |b| {
        b.iter(|| {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("output.env");
            write_env_file(&file_path, &env_vars, false).unwrap();
            black_box(());
        })
    });
}

fn bench_write_medium_env_file(c: &mut Criterion) {
    let env_vars = generate_large_env_vars(100);

    c.bench_function("write_medium_env_file", |b| {
        b.iter(|| {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("output.env");
            write_env_file(&file_path, &env_vars, false).unwrap();
            black_box(());
        })
    });
}

fn bench_write_large_env_file(c: &mut Criterion) {
    let env_vars = generate_large_env_vars(1000);

    c.bench_function("write_large_env_file", |b| {
        b.iter(|| {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("output.env");
            write_env_file(&file_path, &env_vars, false).unwrap();
            black_box(());
        })
    });
}

fn bench_validate_small_env_file(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("validate_small.env");

    let content = create_env_file_content(10);
    fs::write(&file_path, content).unwrap();

    c.bench_function("validate_small_env_file", |b| {
        b.iter(|| {
            validate_env_file(&file_path).unwrap();
            black_box(());
        })
    });
}

fn bench_validate_large_env_file(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("validate_large.env");

    let content = create_env_file_content(1000);
    fs::write(&file_path, content).unwrap();

    c.bench_function("validate_large_env_file", |b| {
        b.iter(|| {
            validate_env_file(&file_path).unwrap();
            black_box(());
        })
    });
}

fn bench_roundtrip_operations(c: &mut Criterion) {
    let env_vars = generate_large_env_vars(100);

    c.bench_function("roundtrip_operations", |b| {
        b.iter(|| {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("roundtrip.env");

            // Write and read back
            write_env_file(&file_path, &env_vars, false).unwrap();
            black_box(read_env_file(&file_path).unwrap());
        })
    });
}

fn bench_merge_operations(c: &mut Criterion) {
    let base_vars = generate_large_env_vars(50);
    let new_vars = generate_large_env_vars(50);

    c.bench_function("merge_operations", |b| {
        b.iter(|| {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("merge.env");

            // Write base, then merge
            write_env_file(&file_path, &base_vars, false).unwrap();
            write_env_file(&file_path, &new_vars, true).unwrap();
            black_box(());
        })
    });
}

fn bench_complex_env_values(c: &mut Criterion) {
    let mut env_vars = HashMap::new();

    // Add various complex values
    env_vars.insert("JSON_CONFIG".to_string(), r#"{"database":{"host":"localhost","port":5432,"credentials":{"username":"admin","password":"secret123"}},"features":{"logging":true,"debug":false}}"#.to_string());
    env_vars.insert("LONG_URL".to_string(), "https://api.example.com/v1/users/12345/profile?include=settings,preferences,history&format=json&api_key=abcdef123456".to_string());
    env_vars.insert("MULTILINE_SQL".to_string(), "SELECT u.id, u.name, u.email, p.title, p.content FROM users u LEFT JOIN posts p ON u.id = p.user_id WHERE u.active = true AND p.published_at > '2023-01-01' ORDER BY p.published_at DESC".to_string());
    env_vars.insert("BASE64_DATA".to_string(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".to_string());

    // Replicate to make it larger
    for i in 0..100 {
        env_vars.insert(format!("COMPLEX_VAR_{}", i), format!("Complex value with special chars: !@#$%^&*()_+-=[]{{}}|;':\",./<>?`~ and number {}", i));
    }

    c.bench_function("complex_env_values", |b| {
        b.iter(|| {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("complex.env");

            write_env_file(&file_path, &env_vars, false).unwrap();
            black_box(read_env_file(&file_path).unwrap());
        })
    });
}

criterion_group!(
    benches,
    bench_read_small_env_file,
    bench_read_medium_env_file,
    bench_read_large_env_file,
    bench_write_small_env_file,
    bench_write_medium_env_file,
    bench_write_large_env_file,
    bench_validate_small_env_file,
    bench_validate_large_env_file,
    bench_roundtrip_operations,
    bench_merge_operations,
    bench_complex_env_values,
);

criterion_main!(benches);
