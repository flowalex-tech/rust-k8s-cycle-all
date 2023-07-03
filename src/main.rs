extern crate clap;
extern crate core;

use std::io;
use std::io::{BufRead, BufReader, stdout};
use clap::{App, Arg, SubCommand};
use std::process::{Command, Stdio};

fn main() {
    let matches = App::new("cycle_all")
        .version("0.1.0")
        .about("Cycles all k8s deployments based on a parameter")
        .arg(
            Arg::with_name("param")
                .short("p")
                .long("param")
                .takes_value(true)
                .required(true)
                .help("Parameter to cycle on"),
        )
        .arg(Arg::with_name("namespace")
            .short("n")
            .long("namespace")
            .takes_value(true)
            .required(true)
            .help("Namespace to cycle on")
        )
        .arg(Arg::with_name("cluster")
            .short("c")
            .long("cluster")
            .takes_value(true)
            .required(true)
            .help("Cluster to cycle on")
        )
        .get_matches();
    let param = matches.value_of("param").unwrap();
    let namespace = matches.value_of("namespace").unwrap();
    let cluster = matches.value_of("cluster").unwrap();

    println!("Param: {}", param);
    println!("Namespace: {}", namespace);
    println!("Cluster: {}", cluster);

    let mut update_context = Command::new("kubectl")
        .arg("config")
        .arg("use-context")
        .arg(cluster)
        .stdout(Stdio::piped())
        .spawn().expect("Failed to change context");

    update_context.wait().expect("Failed to change context");

    let directory = std::env::current_dir().unwrap();
    let mut kubectl_output_child = Command::new("kubectl")
        .arg("get")
        .arg("deployments")
        .arg("--namespace")
        .arg(namespace)
        .stdout(Stdio::piped())
        .spawn().expect("Failed to get deployments");;

    if let Some(mut kubectl_output) = kubectl_output_child.stdout.take() {
        let mut sort_output_child = Command::new("grep")
            .arg(param)
            .stdin(kubectl_output)
            .spawn().expect("Failed to filter deployments");

        kubectl_output_child.wait().expect("Failed to filter deployments");

        if let Some(sort_output) = sort_output_child.stdout.take() {
            let mut head_output_child = Command::new("cut")
                .arg(&["-d", " "])
                .arg(&["-f", " "])
                .stdin(sort_output)
                .stdout(Stdio::piped())
                .spawn().expect("Failed to sort deployments");


            sort_output_child.wait().expect("Failed to sort deployments");

            let deployment_list_child = head_output_child.stdout.take().unwrap();

            let deployment_list = BufReader::new(deployment_list_child).lines();

            for mut line in deployment_list {

                let cycle_deployment = Command::new("kubectl")
                    .arg("rollout")
                    .arg("restart")
                    .arg("deployment")
                    .arg(line.as_mut().unwrap())
                    .arg("--namespace")
                    .arg(namespace)
                    .stdout(Stdio::piped())
                    .spawn().expect("Failed to restart deployment");

                cycle_deployment.wait_with_output().expect("Failed to restart deployment");

                println!("Restarted deployment: {}", line.unwrap());

                head_output_child.wait().expect("head sort failed")
            }
        }
    }
}