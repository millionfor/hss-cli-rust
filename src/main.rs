/**
 * @FileName        hss-cli-rust/src/main.rs
 * @CreatedTime     五, 06 20, 2025 10:07:19 CST
 * @LastModified    五, 06 20, 2025 14:50:19 CST
 * @Author          QuanQuan <millionfor@apache.org>
 * @Description     {{FILER}}
 */

use jenkins_sdk::{
    JenkinsAsync,
    StopBuild,
    core::{
        QueueLength,
        JobDetail,
        LastBuildConsole,
        LastBuildInfo,
        TriggerBuild,
    }
};
use serde_json::Value;
use tokio;
use std::time::Duration;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::collections::HashMap;

// 读取现有配置文件
fn read_config() -> io::Result<HashMap<String, String>> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "找不到主目录"))?;
    let mut config_path = PathBuf::from(home_dir);
    config_path.push(".hssrc");
    let mut config = HashMap::new();
    if config_path.exists() {
        let file = fs::File::open(&config_path)?;
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line = line?.trim().to_string();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                // 只保留第一个 jenkins_url/user/token
                if (key == "jenkins_url" || key == "user" || key == "token") && !config.contains_key(key) {
                    config.insert(key.to_string(), value.to_string());
                }
            }
        }
    }
    Ok(config)
}

// 写入新的配置
fn write_config(jenkins_url: &str, user: &str, token: &str) -> io::Result<()> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "找不到主目录"))?;

    let mut config_path = PathBuf::from(home_dir);
    config_path.push(".hssrc");

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(config_path)?;

    writeln!(file, "jenkins_url={}", jenkins_url)?;
    writeln!(file, "user={}", user)?;
    writeln!(file, "token={}", token)?;

    Ok(())
}

// Jenkins 构建监控相关函数
async fn check_queue_status(client: &JenkinsAsync) -> Result<Value, Box<dyn std::error::Error>> {
    let queue_info = client.request(&QueueLength).await?;
    Ok(queue_info)
}

async fn get_job_details(client: &JenkinsAsync, job_name: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let job_info = client.request(&JobDetail(job_name)).await?;
    Ok(job_info)
}

async fn monitor_build(
    client: &JenkinsAsync,
    job_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_console_size = 0;
    
    loop {
        let build_info = client.request(&LastBuildInfo(job_name)).await?;
        let console_output = client.request(&LastBuildConsole(job_name)).await?;
        
        if console_output.len() > last_console_size {
            print!("{}", &console_output[last_console_size..]);
            last_console_size = console_output.len();
        }
        
        let is_building = build_info.get("building")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
            
        if !is_building {
            let result = build_info.get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");
            println!("\n构建完成，结果: {}", result);
            break;
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    Ok(())
}

async fn jenkins_stop(
    job_name: &str,
    branch: &str,
    env: &str,
    jenkins_url: &str,
    user: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = JenkinsAsync::builder(jenkins_url)
        .auth_basic(user, token)
        .build();

    // 获取作业详情
    let job_info = get_job_details(&client, job_name).await?;

    // 获取最后一次构建编号并尝试停止它
    if let Some(build_number) = job_info
        .get("lastBuild")
        .and_then(|build| build.get("number"))
        .and_then(|v| v.as_u64())
    {
        // 将构建编号转换为字符串
        let build_id = build_number.to_string();
        
        // 停止构建，需要传入 &str
        let resp: String = client
            .request(&StopBuild {
                job: job_name,
                build: &build_id,
            })
            .await?;

        println!("构建已停止");
    } else {
        eprintln!("找不到最后的构建编号，无法停止构建。");
    }

    Ok(())
}

async fn trigger_and_monitor_build(
    job_name: &str,
    branch: &str,
    env: &str,
    jenkins_url: &str,
    user: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = JenkinsAsync::builder(jenkins_url)
        .auth_basic(user, token)
        .build();
    
    // 1. 检查队列状态函数
    async fn check_queue_status(client: &JenkinsAsync) -> Result<Value, Box<dyn std::error::Error>> {
        let queue_info = client.request(&QueueLength).await?;
        Ok(queue_info)
    }
    
    // 2. 获取作业详情函数
    async fn get_job_details(client: &JenkinsAsync, job_name: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let job_info = client.request(&JobDetail(job_name)).await?;
        Ok(job_info)
    }
    
    // 3. 构建监控函数
    async fn monitor_build(
        client: &JenkinsAsync,
        job_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_console_size = 0;
        
        loop {
            // 获取构建状态
            let build_info = client.request(&LastBuildInfo(job_name)).await?;
            
            // 获取控制台输出
            let console_output = client.request(&LastBuildConsole(job_name)).await?;
            
            // 只输出新的日志内容
            if console_output.len() > last_console_size {
                print!("{}", &console_output[last_console_size..]);
                last_console_size = console_output.len();
            }
            
            // 安全地检查构建状态
            let is_building = build_info.get("building")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
                
            if !is_building {
                let result = build_info.get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNKNOWN");
                println!("\n构建完成，结果: {}", result);
                break;
            }
            
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        
        Ok(())
    }
    
    // 4. 主要执行流程
    println!("开始检查构建队列...");
    
    // 触发新的构建
    println!("正在触发构建...");
    let params = serde_json::json!({
        "hydee_git_branch": branch,
        "env": env
    });
    
    client.request(&TriggerBuild {
        job: job_name,
        params: &params,
    }).await?;
    
    println!("构建已触发，等待构建开始...");
    
    // 等待一小段时间确保构建已进入队列
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // 修改后的构建状态监控逻辑
    let mut build_started = false;
    let mut previous_build_number = None;
    
    while !build_started {
        let job_info = get_job_details(&client, job_name).await?;
        
        // 获取最后一次构建编号
        let current_build_number = job_info
            .get("lastBuild")
            .and_then(|build| build.get("number"))
            .and_then(|v| v.as_u64());
            
        // 如果是新的构建编号
        if let Some(current_number) = current_build_number {
            if previous_build_number.is_none() {
                previous_build_number = Some(current_number);
                println!("等待构建开始，构建编号: {}", current_number);
            } else if current_number > previous_build_number.unwrap() {
                // 检查新构建的状态
                let build_info = client.request(&LastBuildInfo(job_name)).await?;
                let is_building = build_info.get("building")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                    
                if is_building {
                    println!("构建已开始执行！构建编号: {}", current_number);
                    build_started = true;
                }
            }
        }
        
        if !build_started {
            println!("等待构建开始...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
    
    // 开始监控构建过程
    println!("开始监控构建过程...");
    monitor_build(&client, job_name).await?;
    
    Ok(())

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("用法：");
        eprintln!("登录: hss-cli login <jenkins_url> <user> <token>");
        eprintln!("构建: hss-cli build <project> <branch> <env>");
        return Ok(());
    }

    let command = &args[1];
    
    match command.as_str() {
        "login" => {
            if args.len() != 5 {
                eprintln!("用法: hss-cli login <jenkins_url> <user> <token>");
                return Ok(());
            }
            
            let jenkins_url = &args[2];
            let user = &args[3];
            let token = &args[4];
            
            if let Err(e) = write_config(jenkins_url, user, token) {
                eprintln!("配置写入失败: {}", e);
            } else {
                println!("配置已保存:");
                println!("jenkins_url={}", jenkins_url);
                println!("user={}", user);
                println!("token={}", token);
            }
        },
        
        "build" => {
            if args.len() != 5 {
                eprintln!("用法: hss-cli build <project> <branch> <env>");
                return Ok(());
            }
            
            let config = read_config()?;
            let (jenkins_url, user, token) =
                match (
                    config.get("jenkins_url"),
                    config.get("user"),
                    config.get("token"),
                ) {
                    (Some(j), Some(u), Some(t)) => (j.as_str(), u.as_str(), t.as_str()),
                    _ => {
                        eprintln!("请先运行 'hss-cli login <jenkins_url> <user> <token>' 进行登录");
                        return Ok(());
                    }
                };

            // println!("读取到的 user: {}", user);
            // println!("读取到的 token: {}", token);

            let project = &args[2];
            let branch = &args[3];
            let env = &args[4];
            
            tokio::select! {
                res = trigger_and_monitor_build(project, branch, env, jenkins_url, user, token) => {
                    if let Err(e) = res {
                        eprintln!("构建过程中发生错误: {}", e);
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    // println!("\n收到 Ctrl+C 信號...");
                    if let Err(e) = jenkins_stop(project, branch, env, jenkins_url, user, token).await {
                        eprintln!("停止 Jenkins 构建时发生错误: {}", e);
                    }
                    // println!("\n程序已退出。");
                }
            }
        },
        
        _ => {
            eprintln!("未知命令: {}", command);
            eprintln!("可用命令: login, build");
        }
    }
    
    Ok(())
}

// vim: set ft=rust fdm=marker et ff=unix tw=180 sw=2:
