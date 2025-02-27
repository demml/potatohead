if __name__ == "__main__":
    from potato_head import Message, ChatPrompt, Mouth, OpenAIConfig

    mouth = Mouth(OpenAIConfig())

    prompt = ChatPrompt(
        model="gpt-4o",
        messages=[
            Message("user", "What is 4 + 1?"),
        ],
    )

    # save_path
    save_path = prompt.save_prompt()

    # load from path
    loaded_prompt = ChatPrompt.load_from_path(save_path)

    response = mouth.speak(loaded_prompt)
    print(response)
