use crate::common::{create_anthropic_prompt, create_google_prompt, create_openai_prompt};
use baked_potato::{create_parameterized_prompt, LLMTestServer};
use potato_agent::TaskStatus;
use potato_agent::{Agent, Task};
use potato_type::prompt::MessageNum;
use potato_type::prompt::Role;
use potato_type::{Provider, StructuredOutput};
use potato_workflow::Workflow;
use schemars::JsonSchema;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, JsonSchema, Deserialize)]
struct Parameters {
    variable1: i32,
    variable2: i32,
}
impl StructuredOutput for Parameters {}

#[test]
fn test_workflow() {
    // Test workflow with multiple agents and tasks with dependencies
    // Setup:
    // - Create two agents
    // - Create 2 tasks for agent1 and 2 tasks for agent2
    // - Task3 depends on Task1 and Task2
    // - Task4 depends on Task3
    // - Final task depends on Task3 and Task4
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let prompt = create_openai_prompt(None);
    let mut workflow = Workflow::new("My Workflow");

    let agent1 = runtime
        .block_on(async { Agent::new(Provider::OpenAI, None).await })
        .unwrap();
    let agent2 = runtime
        .block_on(async { Agent::new(Provider::OpenAI, None).await })
        .unwrap();

    workflow.add_agent(&agent1);
    workflow.add_agent(&agent2);

    // add a task to the workflow
    workflow
        .add_task(Task::new(&agent1.id, prompt.clone(), "task1", None, None).unwrap())
        .unwrap();
    workflow
        .add_task(Task::new(&agent2.id, prompt.clone(), "task2", None, None).unwrap())
        .unwrap();
    workflow
        .add_task(
            Task::new(
                &agent2.id,
                prompt.clone(),
                "task3",
                Some(vec!["task1".to_string(), "task2".to_string()]),
                None,
            )
            .unwrap(),
        )
        .unwrap();
    workflow
        .add_task(
            Task::new(
                &agent1.id,
                prompt.clone(),
                "task4",
                Some(vec!["task3".to_string()]),
                None,
            )
            .unwrap(),
        )
        .unwrap();

    // add final task
    workflow
        .add_task(
            Task::new(
                &agent1.id,
                prompt.clone(),
                "final_task",
                Some(vec!["task3".to_string(), "task4".to_string()]),
                None,
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(workflow.task_list.len(), 5);
    assert!(!workflow.task_list.is_empty());

    // run the workflow and check
    let workflow_result = runtime.block_on(async { workflow.run(None).await.unwrap() });
    let workflow_result = workflow_result.read().unwrap();

    // Get task references for context verification
    let task1 = workflow_result.task_list.get_task("task1").unwrap();
    let task2 = workflow_result.task_list.get_task("task2").unwrap();
    let task3 = workflow_result.task_list.get_task("task3").unwrap();
    let task4 = workflow_result.task_list.get_task("task4").unwrap();
    let final_task = workflow_result.task_list.get_task("final_task").unwrap();

    // Verify all tasks have results
    assert!(
        task1.read().unwrap().result.is_some(),
        "task1 should have a result"
    );
    assert!(
        task2.read().unwrap().result.is_some(),
        "task2 should have a result"
    );
    assert!(
        task3.read().unwrap().result.is_some(),
        "task3 should have a result"
    );
    assert!(
        task4.read().unwrap().result.is_some(),
        "task4 should have a result"
    );
    assert!(
        final_task.read().unwrap().result.is_some(),
        "final_task should have a result"
    );

    // Verify task statuses
    assert_eq!(task1.read().unwrap().status, TaskStatus::Completed);
    assert_eq!(task2.read().unwrap().status, TaskStatus::Completed);
    assert_eq!(task3.read().unwrap().status, TaskStatus::Completed);
    assert_eq!(task4.read().unwrap().status, TaskStatus::Completed);
    assert_eq!(final_task.read().unwrap().status, TaskStatus::Completed);

    // ===== Context Verification =====

    // Task3 should have context from task1 and task2
    // During execution, dependency messages are inserted before the first user message
    let task3_unwrapped = task3.read().unwrap();
    let task3_messages = task3_unwrapped.prompt.request.messages();
    let task3_msg_count = task3_messages.len();
    let original_msg_count = prompt.request.messages().len();

    // Task3 should have 2 additional messages (from task1 and task2)
    assert_eq!(
        task3_msg_count,
        original_msg_count + 2,
        "task3 should have {} messages ({} original + 2 dependency contexts)",
        original_msg_count + 2,
        original_msg_count
    );

    // Verify the dependency messages are assistant messages (the response from dependencies)
    let task3_assistant_messages: Vec<_> = task3_messages
        .iter()
        .filter(|msg| msg.role() == Role::Assistant.as_str())
        .collect();
    assert_eq!(
        task3_assistant_messages.len(),
        2,
        "task3 should have 2 assistant messages from dependencies"
    );

    // Task4 should have context from task3
    let task4_unqrapped = task4.read().unwrap();
    let task4_messages = task4_unqrapped.prompt.request.messages();
    let task4_msg_count = task4_messages.len();

    // Task4 should have 1 additional message (from task3)
    assert_eq!(
        task4_msg_count,
        original_msg_count + 1,
        "task4 should have {} messages ({} original + 1 dependency context)",
        original_msg_count + 1,
        original_msg_count
    );

    let task4_assistant_messages: Vec<_> = task4_messages
        .iter()
        .filter(|msg| msg.role() == Role::Assistant.as_str())
        .collect();
    assert_eq!(
        task4_assistant_messages.len(),
        1,
        "task4 should have 1 assistant message from dependency"
    );

    // Final task should have context from task3 and task4
    let final_unwrapped = final_task.read().unwrap();
    let final_messages = final_unwrapped.prompt.request.messages();
    let final_msg_count = final_messages.len();

    // Final task should have 2 additional messages (from task3 and task4)
    assert_eq!(
        final_msg_count,
        original_msg_count + 2,
        "final_task should have {} messages ({} original + 2 dependency contexts)",
        original_msg_count + 2,
        original_msg_count
    );

    let final_assistant_messages: Vec<_> = final_messages
        .iter()
        .filter(|msg| msg.role() == Role::Assistant.as_str())
        .collect();
    assert_eq!(
        final_assistant_messages.len(),
        2,
        "final_task should have 2 assistant messages from dependencies"
    );

    // Verify message order: system messages first, then dependency contexts, then user message
    // This follows the logic in Agent::append_task_with_message_dependency_context
    for (task_name, messages) in [
        ("task3", task3_messages),
        ("task4", task4_messages),
        ("final_task", final_messages),
    ] {
        let first_user_idx = messages
            .iter()
            .position(|msg| msg.role() == Role::User.as_str())
            .unwrap_or_else(|| panic!("{} should have a user message", task_name));

        // All assistant messages (dependency context) should come before the user message
        for (idx, msg) in messages.iter().enumerate() {
            if msg.role() == Role::Assistant.as_str() {
                assert!(
                    idx < first_user_idx,
                    "{}: assistant message at index {} should come before user message at index {}",
                    task_name,
                    idx,
                    first_user_idx
                );
            }
        }
    }

    // serialize the workflow
    let serialized = workflow.serialize().unwrap();
    let mut reloaded = Workflow::from_json(&serialized).unwrap();

    // before running reset agents assert clients are undefined
    for agent in reloaded.agents.values() {
        assert!(agent.client_provider() == &Provider::Undefined);
    }

    runtime.block_on(async {
        reloaded.reset_agents().await.unwrap();
        reloaded.run(None).await.unwrap();
    });

    // assert workflow agent client are not undefined
    for agent in reloaded.agents.values() {
        assert!(agent.client_provider() == &Provider::OpenAI);
    }

    mock.stop_server().unwrap();
}

#[test]
fn test_parameterized_workflow() {
    // Flow:
    // - Create a workflow with two tasks
    // - The second task uses a parameterized prompt with variables from the first task's output
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let prompt = create_openai_prompt(Some(Parameters::get_structured_output_schema()));
    let parameterized_prompt = create_parameterized_prompt();

    // assert 2 variables are in the prompt
    assert_eq!(parameterized_prompt.parameters.len(), 2);

    let mut workflow = Workflow::new("My Workflow");
    let agent1 = runtime
        .block_on(async { Agent::new(Provider::OpenAI, None).await })
        .unwrap();
    workflow.add_agent(&agent1);

    workflow
        .add_task(Task::new(&agent1.id, prompt.clone(), "task1", None, None).unwrap())
        .unwrap();

    workflow
        .add_task(
            Task::new(
                &agent1.id,
                parameterized_prompt.clone(),
                "task2",
                Some(vec!["task1".to_string()]),
                None,
            )
            .unwrap(),
        )
        .unwrap();

    // assert pending task count is
    assert_eq!(workflow.pending_count(), 2);

    let result = runtime.block_on(async { workflow.run(None).await.unwrap() });

    // assert original workflow is unmodified
    assert_eq!(workflow.task_list.len(), 2);
    assert_eq!(workflow.pending_count(), 2);
    // assert result.total_duration is greater than 0
    assert!(result.read().unwrap().total_duration() > 0);

    // assert result id is not the same as workflow id
    assert_ne!(result.read().unwrap().id, workflow.id);

    let task1_output = result
        .read()
        .unwrap()
        .task_list
        .get_task("task1")
        .unwrap()
        .read()
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .response_text();

    // validate task1_output can be deserialized into Parameters struct
    // Should be a structured output JSON
    let _ = Parameters::model_validate_json_str(&task1_output);

    // assert original workflow is unmodified
    assert_eq!(workflow.task_list.len(), 2);

    // assert workflow event tracker is empty for the original workflow
    assert!(workflow.event_tracker.read().unwrap().is_empty());

    // assert the recent run workflow has events
    assert!(!result
        .read()
        .unwrap()
        .event_tracker
        .read()
        .unwrap()
        .is_empty());

    let binding = result.read().unwrap().task_list.get_task("task2").unwrap();
    let binding = binding.read().unwrap();
    let task2_output = binding.prompt.request.messages();

    // assert task2_output len is 3
    // (1 developer message + 1 assistant message from task1 + 1 user message)
    assert_eq!(task2_output.len(), 3);

    let serialized = workflow.serialize().unwrap();

    let _deserialized: Workflow = serde_json::from_str(&serialized).unwrap();

    // call workflow.execute_task directly for task1
    let empty_context = serde_json::json!({});
    let task1 = runtime.block_on(async {
        workflow
            .execute_task("task1", &empty_context)
            .await
            .unwrap()
    });

    let task1_value_orig = result
        .read()
        .unwrap()
        .task_list
        .get_task("task1")
        .unwrap()
        .read()
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .response_value()
        .unwrap();

    // assert task1 result is same as previous task1 result
    assert_eq!(task1, task1_value_orig,);
}

#[test]
fn test_vendor_switching() {
    // Flow:
    // 3 tasks - 1 OpenAI -> 2 Anthropic -> 3 Gemini (depends on 1 and 2)
    // OpenAI tasks output should be converted to Anthropic input
    //  Anthropic tasks output should be converted to Gemini input
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut mock = LLMTestServer::new();
    mock.start_server().unwrap();

    let mut workflow = Workflow::new("Vendor Workflow");

    let openai_agent = runtime
        .block_on(async { Agent::new(Provider::OpenAI, None).await })
        .unwrap();
    let anthropic_agent = runtime
        .block_on(async { Agent::new(Provider::Anthropic, None).await })
        .unwrap();
    let gemini_agent = runtime
        .block_on(async { Agent::new(Provider::Gemini, None).await })
        .unwrap();

    // create tasks
    workflow.add_agents(&[&openai_agent, &anthropic_agent, &gemini_agent]);

    workflow
        .add_task(
            Task::new(
                &openai_agent.id,
                create_openai_prompt(None),
                "openai_task",
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap();

    workflow
        .add_task(
            Task::new(
                &anthropic_agent.id,
                create_anthropic_prompt(),
                "anthropic_task",
                Some(vec!["openai_task".to_string()]),
                None,
            )
            .unwrap(),
        )
        .unwrap();

    workflow
        .add_task(
            Task::new(
                &gemini_agent.id,
                create_google_prompt(),
                "gemini_task",
                Some(vec![
                    "openai_task".to_string(),
                    "anthropic_task".to_string(),
                ]),
                None,
            )
            .unwrap(),
        )
        .unwrap();

    // This will fail if the message conversions do not work correctly
    let result = runtime.block_on(async { workflow.run(None).await.unwrap() });
    let workflow_result = result.read().unwrap();

    // Verify all tasks completed
    assert!(workflow_result.is_complete());

    // Get task references
    let openai_task = workflow_result.task_list.get_task("openai_task").unwrap();
    let anthropic_task = workflow_result
        .task_list
        .get_task("anthropic_task")
        .unwrap();
    let gemini_task = workflow_result.task_list.get_task("gemini_task").unwrap();

    // ===== Verify OpenAI Task Messages =====
    let openai_unwrapped = openai_task.read().unwrap();
    let openai_messages = openai_unwrapped.prompt.request.messages();

    // All messages should be OpenAI type
    for (idx, msg) in openai_messages.iter().enumerate() {
        assert!(
            matches!(msg, MessageNum::OpenAIMessageV1(_)),
            "openai_task message at index {} should be OpenAIMessageV1, got: {:?}",
            idx,
            msg
        );
    }

    // Verify at least has developer + user messages
    assert!(
        openai_messages.len() >= 2,
        "openai_task should have at least 2 messages"
    );

    // ===== Verify Anthropic Task Messages =====
    let anthropic_unwrapped = anthropic_task.read().unwrap();
    let anthropic_messages = anthropic_unwrapped.prompt.request.messages();

    // All messages should be Anthropic type (including converted OpenAI dependency)
    for (idx, msg) in anthropic_messages.iter().enumerate() {
        assert!(
            matches!(msg, MessageNum::AnthropicMessageV1(_)),
            "anthropic_task message at index {} should be AnthropicMessageV1, got: {:?}",
            idx,
            msg
        );
    }

    // Should have: original user message + 1 assistant message from openai_task dependency
    assert_eq!(
        anthropic_messages.len(),
        2,
        "anthropic_task should have 2 messages (1 original + 1 from dependency)"
    );

    // Verify the dependency context (assistant message) was added
    let anthropic_assistant_count = anthropic_messages
        .iter()
        .filter(|msg| msg.role() == Role::Assistant.as_str())
        .count();
    assert_eq!(
        anthropic_assistant_count, 1,
        "anthropic_task should have 1 assistant message from openai_task dependency"
    );

    // ===== Verify Gemini Task Messages =====
    let gemini_unwrapped = gemini_task.read().unwrap();
    let gemini_messages = gemini_unwrapped.prompt.request.messages();

    // All messages should be Gemini type (including converted dependencies)
    for (idx, msg) in gemini_messages.iter().enumerate() {
        assert!(
            matches!(msg, MessageNum::GeminiContentV1(_)),
            "gemini_task message at index {} should be GeminiContentV1, got: {:?}",
            idx,
            msg
        );
    }

    // Should have: original user message + 2 assistant messages from dependencies
    assert_eq!(
        gemini_messages.len(),
        3,
        "gemini_task should have 3 messages (1 original + 2 from dependencies)"
    );

    // Verify both dependency contexts (assistant messages) were added
    let gemini_assistant_count = gemini_messages
        .iter()
        .filter(|msg| msg.role() == Role::Assistant.as_str())
        .count();
    assert_eq!(
        gemini_assistant_count, 2,
        "gemini_task should have 2 assistant messages from dependencies"
    );

    // ===== Verify Message Order =====
    // For each task, verify assistant messages (dependencies) come before user messages
    for (task_name, messages) in [
        ("anthropic_task", anthropic_messages),
        ("gemini_task", gemini_messages),
    ] {
        let first_user_idx = messages
            .iter()
            .position(|msg| msg.role() == Role::User.as_str())
            .unwrap_or_else(|| panic!("{} should have a user message", task_name));

        for (idx, msg) in messages.iter().enumerate() {
            if msg.role() == Role::Assistant.as_str() {
                assert!(
                    idx < first_user_idx,
                    "{}: assistant message at index {} should come before user message at index {}",
                    task_name,
                    idx,
                    first_user_idx
                );
            }
        }
    }

    mock.stop_server().unwrap();
}
