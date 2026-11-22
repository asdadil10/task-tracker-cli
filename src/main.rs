use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

//todo: implement timezone setup automatically

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
#[derive(Serialize, Deserialize, Hash)]
pub enum Status {
    RUNNING,
    PAUSED,
    TODO,
    COMPLETED,
}
impl Status {
    fn to_string(&self) -> String {
        match self {
            Status::RUNNING => "Running".to_string(),
            Status::PAUSED => "Paused".to_string(),
            Status::TODO => "Todo".to_string(),
            Status::COMPLETED => "Completed".to_string(),
        }
    }
}
#[derive(Serialize, Deserialize, Hash)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub tag: Option<String>,
    pub time: u64,
    pub status: Status,
    pub started_at: Option<u64>,
}
fn get_formated_time(time_in_s: &u64, duration: bool) -> String {
    if duration {
        format!(
            "{:>3}H:{:2}M:{:2}S",
            time_in_s / 3600,
            (time_in_s / 60) % 60,
            time_in_s % 60
        )
        .to_string()
    } else {
        format!(
            "{:2}:{:0>2}:{:<2}",
            (time_in_s / 3600) % 24,
            (time_in_s / 60) % 60,
            time_in_s % 60
        )
        .to_string()
    }
}
impl Task {
    // ID | TIME | STATUS | NAME
    // Note : in list you will sort by tag so no need to redisplay tag
    fn to_string(&self) -> String {
        //todo: implement overflow coping logic for names and tags
        format!(
            " {:<3} |{:12}|{} | {:40} | {}",
            self.id,
            self.tag.as_deref().unwrap_or("No Tag"),
            get_formated_time(&self.time, true),
            self.name,
            if matches!(self.status, Status::RUNNING) {
                format!(
                    "Start: {} UTC",
                    get_formated_time(&self.started_at.unwrap_or(0), false)
                )
            } else {
                self.status.to_string()
            },
        )
        .to_string()
    }
}
#[derive(Serialize, Deserialize, Default, Hash)]
pub struct TaskStore {
    pub next_id: u32,
    pub tasks: Vec<Task>,
}
const APP_VERSION: &str = "0.1.0";
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut store = load_store().unwrap_or_default();
    let before = calculate_hash(&store);
    // args[0] is always the path to the binary itself
    // User-supplied arguments start at index 1
    if args.len() < 2 {
        println!("No argument provided...");
        help(&args)
    } else if !help_asked(&args) {
        match args[1].as_str() {
            "add" => add(&args, &mut store),
            "start" => start(&args, &mut store),
            "stop" => {
                check_len(&args, 2);
                if let Some(id) = stop(&mut store) {
                    println!("1 Task stopped. ID \"{}\"", id);
                } else {
                    eprintln!("No task currently running!");
                };
            }
            "done" => done(&args, &mut store),
            "list" => list(&args, &store),
            "summary" => summary(&args, &store),
            "delete" => {
                if let Some(id) = delete(&args, &mut store) {
                    println!("Task deleted with ID \"{}\"", id);
                } else {
                    eprintln!("No task currently running!");
                }
            }
            "help" | "-h" | "--help" => help(&args),
            "-v" | "--version" => println!("tt v{}", APP_VERSION),
            _ => println!("Unknown command: {}", args[1]),
        }
        if calculate_hash(&store) != before {
            if let Err(err) = save_store(&store) {
                eprintln!("Error saving tasks: {err}");
            }
        }
    }
}

fn load_store() -> Option<TaskStore> {
    println!("Loading store...");
    // file reading
    let data = fs::read_to_string("tasks.json").ok()?;
    let loaded_store: TaskStore = serde_json::from_str(&data).ok()?;
    println!("Loaded: From tasks.json");
    Some(loaded_store)
}
fn save_store(store: &TaskStore) -> Result<(), Box<dyn std::error::Error>> {
    println!("Saving...");
    let data = serde_json::to_string(&store)?;
    fs::write("tasks.json", data)?;
    Ok(())
}

fn help_text() -> String {
    "USAGE:
        tt <COMMAND>

    COMMANDS:
        add       Add a new task
        start     Start tracking time on a task (pauses any active task)
        stop      Stop the currently running timer
        done      Mark a task as completed
        list      Display tasks and logged time
        summary   Show total time grouped by tag
        delete    Remove a task
        help      Print this message or the help of the given subcommand(s)

    OPTIONS:
        -h, --help       Print help
        -v, --version    Print version

    EXAMPLES:
        tt add \"Fix login bug\" --tag backend
        tt start 1
        tt list --tag backend
        tt done 1"
        .to_string()
}

// if user writes ./tt help [cmd]
fn help(args: &Vec<String>) {
    let target = match (
        args.get(1).map(|x| x.as_str()),
        args.get(2).map(|x| x.as_str()),
    ) {
        (Some("help"), Some(cmd)) => cmd,
        (Some(cmd), Some("-h" | "--help")) => cmd,
        (Some("-h" | "--help" | "help"), None) | (None, None) => "",
        (_, Some(cmd)) => cmd,
        (Some(cmd), None) => cmd,
    };
    if args.len() > 3 {
        eprintln!("Too many arguments for help.");
        std::process::exit(1);
    }
    // let subcommand = args.get(if args.len() > 2 {2} else {1}).map(|x| x.as_str()).unwrap_or("");
    match target {
        "add" => println!("Usage: tt add <TITLE> [--tag | -t <TAG>]\nAdd a new task."),
        "start" => println!(
            "Usage: tt start <ID>\nStart tracking time on a task (pauses any active task)."
        ),
        "stop" => println!("Usage: tt stop\nStop the currently running timer."),
        "done" => println!("Usage: tt done <ID>\nMark a task as completed."),
        "list" => {
            println!("Usage: tt list [--all] [--tag | -t <TAG>]\nDisplay tasks and logged time.")
        }
        "summary" => println!("Usage: tt summary [-a | --all]\nShow total time grouped by tag."),
        "delete" => println!("Usage: tt delete <ID>\nRemove a task."),
        "--help" | "-h" | "help" | "" => println!("{}", help_text()),
        _ => {
            eprintln!("Unknown command: {}", target);
            std::process::exit(1);
        }
    }
}

// helper to print help for a specific
fn help_for(name: String) {
    help(&["".to_string(), name].to_vec())
}

// if user writes ./tt cmd [-h | --help]
fn help_asked(args: &Vec<String>) -> bool {
    let mut asked = false;
    let task = args.get(2).map(|s| s.as_str()).unwrap_or("");
    if (task == "-h") | (task == "--help") {
        help(&args);
        asked = true;
    };
    asked
}

// if arg.len() > max exit the program
fn check_len(args: &[String], max_len: usize) {
    if args.len() > max_len {
        eprintln!("Too many arguments for {}.", args[1]);
        help_for(args[1].to_string());
        std::process::exit(1);
    }
}
// add       Add a new task
fn add(args: &Vec<String>, store: &mut TaskStore) {
    // total acceptable args are 5 : ./tt add "task name" -t backend
    // if len > 5 exit automatically
    check_len(&args, 5);
    // 1. Task Name
    let name = args.get(2).map(|s| s.as_str()).unwrap_or("").to_string();
    if name.is_empty() {
        eprintln!("Please specify a task to add!");
        help(&args);
        return;
    }
    // 2. Tag
    let tag_arg = args.get(3).map(|s| s.as_str()).unwrap_or("");
    // get a tag as an Option
    let tag: Option<String> = match tag_arg {
        // if tag is explicitly passed
        "--tag" | "-t" => args.get(4).cloned(),
        //if -t | --tag is not passed
        "" => None,
        // if something trails the task name
        _ => {
            eprintln!("Unknown argument: {}", tag_arg);
            help(&args);
            return;
        }
    };
    // 3. Add to the database
    store.next_id += 1;
    store.tasks.push(Task {
        id: store.next_id,
        name,
        tag,
        time: 0,
        status: Status::TODO,
        started_at: None,
    });
    //println!("{:?}",store.tasks);
}
// start     Start tracking time on a task (pauses any active task)
fn start(args: &Vec<String>, store: &mut TaskStore) {
    check_len(args, 3);
    let id = args.get(2).map(|s| s.as_str()).unwrap_or("").to_string();
    if let Ok(target_id) = id.parse::<u32>() {
        if let Some(index) = store.tasks.iter().position(|x| x.id == target_id) {
            if store.tasks[index].started_at.is_some() {
                eprintln!("Task \"{}\" already started!", id);
            } else if matches!(store.tasks[index].status, Status::COMPLETED) {
                eprintln!("Task \"{}\" already completed!", id);
            } else {
                stop(store);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                store.tasks[index].started_at = Some(now);
                store.tasks[index].status = Status::RUNNING;
            }
        } else {
            eprintln!("Task \"{}\" does not exist!", id);
        }
    } else if id == "" {
        eprintln!("Please specify a task to start!");
    } else {
        eprintln!("Invalid task \"{}\"", id);
    }
}
// stop      Stop the currently running timer
fn stop(store: &mut TaskStore) -> Option<u32> {
    for task in store.tasks.iter_mut() {
        if matches!(task.status, Status::RUNNING) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if let Some(started_at) = task.started_at.take() {
                task.time += now.saturating_sub(started_at);
            }
            task.status = Status::PAUSED;
            return Some(task.id);
        }
    }
    None
}
// done      Mark a task as completed
fn done(args: &Vec<String>, store: &mut TaskStore) {
    check_len(args, 3);
    let id = args.get(2).map(|s| s.as_str()).unwrap_or("").to_string();
    if let Ok(target_id) = id.parse::<u32>() {
        for task in store.tasks.iter_mut() {
            if task.id == target_id {
                if matches!(task.status, Status::COMPLETED) {
                    eprintln!("Task \"{}\" already completed!", id);
                    return;
                }
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if matches!(task.status, Status::RUNNING) {
                    if let Some(started_at) = task.started_at.take() {
                        task.time += now.saturating_sub(started_at);
                    }
                }
                task.status = Status::COMPLETED;
                return;
            }
        }
    } else {
        eprintln!("Task \"{}\" does not exist!", id);
    }
}
// list      Display tasks and logged time
fn list(args: &Vec<String>, store: &TaskStore) {
    check_len(args, 4);
    println!(
        "  ID |    Tag     |   Elapsed   |                Task Name                 |  Status
-----|------------|-------------|------------------------------------------|-----------"
    );
    // Tag
    let tag_arg = args.get(2).map(|s| s.as_str()).unwrap_or("");
    // get a tag as an Option
    match tag_arg {
        // if no --tag arg
        "" => store.tasks.iter().for_each(|task| {
            if !matches!(task.status, Status::COMPLETED) {
                println!("{}", task.to_string())
            }
        }),
        // if tag is explicitly passed
        "--tag" | "-t" => {
            let tag = args.get(3).cloned();
            store.tasks.iter().for_each(|task|
                // either user entered a tag, either not
                if (tag == None && !matches!(task.status, Status::COMPLETED)) || task.tag == tag {
                    println!("{}", task.to_string());
                }
            )
        }
        //if -t | --tag is not passed
        "--all" | "-a" | "a" | "all" => store
            .tasks
            .iter()
            .for_each(|task| println!("{}", task.to_string())),
        // if something trails the task name
        _ => {
            eprintln!("Unknown argument: {}", tag_arg);
            help(&args);
            return;
        }
    };
}
// summary   Show total time grouped by tag
fn summary(args: &[String], store: &TaskStore) {
    check_len(args, 2);
    let mut summary: HashMap<&Option<String>, u64> = HashMap::new();
    for task in store.tasks.iter() {
        // entry() looks up the key in HashMap, if not exists then add entry and set val to 0, then += time
        *summary.entry(&task.tag).or_insert(0) += task.time;
        /*
        if let Some(time) = summary.get_mut(&task.tag) {
            *time += task.time;
        } else {
            summary.insert(task.tag.clone(), task.time);
        }
        */
    }

    let mut entries: Vec<_> = summary.into_iter().collect();

    entries.sort();
    println!(
        " {:^12}  |  {:^10}\n{:-<12}---|----{:-<10}",
        "Tags", "Time", "", ""
    );
    for (tag, time) in entries {
        println!(
            " {:12}  | {}",
            tag.as_deref().unwrap_or("No Tag "),
            get_formated_time(&time, false)
        );
    }

    // todo!("Add -a and --all") change the check len too to 3 then
}
// delete    Remove a task
fn delete(args: &Vec<String>, store: &mut TaskStore) -> Option<u32> {
    check_len(args, 3);
    let id = args.get(2).map(|s| s.as_str()).unwrap_or("").to_string();
    if let Ok(target_id) = id.parse::<u32>() {
        if let Some(index) = store.tasks.iter().position(|t| t.id == target_id) {
            store.tasks.remove(index);
            return Some(target_id);
        }
    }else if id == ""{
        eprintln!("Please specify a task to delete!");
    }
    else {
        eprintln!("Task \"{}\" does not exist!", id);
    }
    None
}
