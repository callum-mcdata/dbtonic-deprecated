import os

output_file_path = 'output.txt'
repo_path = "/Users/callummccann/repos/sqlparser-rs/src"
text_to_remove = """
"""
bad_list = [".DS_Store"]

def process_repository(repo_path, output_file):
    for root, _, files in os.walk(repo_path):
        for file in files:
            if file not in bad_list:
                file_path = os.path.join(root, file)
                relative_file_path = os.path.relpath(file_path, repo_path)
                with open(file_path, 'r', errors='ignore') as file_contents:
                    contents = file_contents.read()
                    output_file.write("-" * 4 + "\n")
                    output_file.write(f"{relative_file_path}\n")
                    output_file.write(f"{contents}\n")

def remove_whitespace(content):
    lines = content.split('\n')
    cleaned_lines = [line.strip() for line in lines]
    cleaned_string = '\n'.join(cleaned_lines)
    return cleaned_string

if __name__ == "__main__":
    with open(output_file_path, 'w') as output_file:
        process_repository(repo_path, output_file)

    # Read the content of the file
    with open(output_file_path, 'r') as output_file:
        file_contents = output_file.read()

    # Replace the specified text
    cleaned_contents = file_contents.replace(text_to_remove, "")

    # Clean the whitespace
    # cleaned_contents = remove_whitespace(cleaned_contents)

    # Write the modified content back to the file
    with open(output_file_path, 'w') as output_file:
        output_file.write(cleaned_contents)
