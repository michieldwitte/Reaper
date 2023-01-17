use std::process::exit;
use std::fs;
use std::env;
use std::ffi::CString;
use std::path::Path;

use libc::{prctl, PR_SET_CHILD_SUBREAPER};
use nix::sys::wait::waitpid;
use nix::unistd::{fork, execv, getpid, ForkResult, Pid};
use nix::sys::signal::{kill, SIGKILL};

fn get_child_pids(process_id: Pid) -> Vec<Pid> {
	let output = fs::read_to_string(format!("/proc/{process_id}/task/{process_id}/children")).expect("Failed to read out file");
	return output.split(" ")
			.filter(|&x| !x.is_empty())
			.map(|x| Pid::from_raw(x.parse::<i32>().unwrap()))
			.collect();
}

fn main() {
	unsafe {
		prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0);
	}

	assert!(env::args().len() > 1, "Not enough arguments given");

	let child_pid = match unsafe{fork()} {
		Ok(ForkResult::Child) => {
			let mut args: Vec<_> = env::args().map(|x| CString::new(x).unwrap()).collect();
			args.remove(0);
			let program = args.remove(0);
			args.insert(0, CString::new(Path::new(&program.to_str().unwrap()).file_name().unwrap().to_str().unwrap()).unwrap());
			execv(&program, &args).expect("Could not run child program");
			exit(0);
		}
		Ok(ForkResult::Parent {child, ..}) => {
			child
		}
		Err(err) => {
			panic!("fork failed: {}", err);
		}
	};
	
	waitpid(child_pid, None).expect(format!("Waiting for main child process pid {child_pid} failed").as_str());
	
	let process_id = getpid();

	loop {
		let child_pids = get_child_pids(process_id);
		if child_pids.len() == 0 { 
			break;
		}
		for child_pid in child_pids {
			match kill(child_pid, SIGKILL) {
				Ok(_) => (),
				Err(error) => {println!("Could not kill pid {}, error: {}", child_pid, error);}
			}
			match waitpid(child_pid, None) {
				Ok(_) => (),
				Err(error) => {println!("Waiting for pid {} failed: {}", child_pid, error);}
			}
		}
	}
}
