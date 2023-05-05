To rewrite the extract_value function to be as idiomatic as possible, I would use an iterator over the characters of the input string. This approach eliminates the need for manually managing indices and can lead to more concise and readable code. Here's an idiomatic version of the extract_value function:

rust
Copy code
fn extract_value(&mut self) -> String {
    let peek_str = |c: char| c.to_string();

    let text: String = self.sql[self.current..]
        .chars()
        .take_while(|&c| c != '\0' && !self.single_tokens.contains_key(&peek_str(c)) && !c.is_whitespace())
        .collect();

    // Update the tokenizer state accordingly
    let text_len = text.chars().count();
    self.advance(text_len);

    text
}
In this version of the function, I've used the take_while method on the character iterator to collect all characters that meet the condition (not a null character, not a key in single_tokens, and not a whitespace character). This iterator-based approach makes the code more concise, readable, and idiomatic.

After collecting the characters into the text string, I update the tokenizer state by calling self.advance with the length of the extracted text. This ensures that the state of the tokenizer remains consistent with the original implementation.