mod persistence;
use persistence::persistence::Task;

use crate::persistence::persistence::Persistable;

fn main() {
    let persistence = persistence::persistence::Persistence::new();
    persistence.sync_schema();

    let task = Task {
        id: Option::None,
        title: "Test task".to_string(),
        description: Some("This is a test task".to_string()),
        completed: true,
    };

    persistence.save(&task);

    let tasks = persistence.get_all::<Task>();
    for task in tasks {
        println!("{}", task.describe());
    }

}
