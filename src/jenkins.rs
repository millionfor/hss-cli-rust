//! jenkins2
//! @CreatedTime     五, 06 20, 2025 19:52:10 CST
//! @LastModified    五, 06 20, 2025 19:52:10 CST
//! @Author          QuanQuan <millionfor@apache.org>
//! @Description     {{FILER}}

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

async fn get_job_details(client: &JenkinsAsync, job_name: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let job_info = client.request(&JobDetail(job_name)).await?;
    Ok(job_info)
}

pub async fn jenkins_stop(
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

// vim: set ft=rs fdm=marker et ff=unix tw=180 sw=2:

