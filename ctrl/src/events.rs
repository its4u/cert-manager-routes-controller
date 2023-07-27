use kube::{runtime::events::{Recorder, Event, EventType}, Error};


/// Publish a successful event
/// 
/// # Arguments
/// 
/// * `action` - The action that was taken
/// * `reason` - The reason for the action
/// * `note` - An optional note to include
/// * `recorder` - The event recorder
pub async fn success_event(
    action: String,
    reason: String,
    note: Option<String>,
    recorder: &Recorder,
) -> (){
    println!("OK; action={}; reason={}; note={:?};", action, reason, note);
    let res = recorder.publish(Event {
        action,
        reason,
        note,
        type_: EventType::Normal,
        secondary: None
    }).await;
    println!("{:?}", res);
    ()
}

/// Publish an unsuccessful event
/// 
/// This event will appear as a warning in the Kubernetes event log
/// 
/// # Arguments
/// 
/// * `action` - The action that was taken
/// * `reason` - The reason for the action
/// * `note` - An optional note to include
/// * `recorder` - The event recorder
pub async fn error_event(
    action: String,
    reason: String,
    note: Option<String>,
    recorder: &Recorder,
) -> () {
    eprintln!("ERR; action={}; reason={}; note={:?};", action, reason, note);
    let _ = recorder.publish(Event {
        action,
        reason,
        note,
        type_: EventType::Warning,
        secondary: None
    }).await;
    ()
}
