import os
from langchain.embeddings import OpenAIEmbeddings
from langchain.embeddings.openai import OpenAIEmbeddings
from langchain.text_splitter import CharacterTextSplitter
from langchain.vectorstores import Chroma
# from tiktoken

from dotenv import load_dotenv

load_dotenv()
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")

with open('output.txt') as f:
    state_of_the_union = f.read()
    text_splitter = CharacterTextSplitter(chunk_size=1000, chunk_overlap=0)
    texts = text_splitter.split_text(state_of_the_union)

breakpoint()
print(texts)

# embeddings = OpenAIEmbeddings(model_name="ada")

# text = "This is a test document."
# query_result = embeddings.embed_query(text)
# doc_result = embeddings.embed_documents([text])
# print(doc_result)

class VectorStore:
    def __init__(self, name: str, docs_loc: str) -> None:
        embedder = OpenAIEmbeddings(client=OPENAI_API_KEY)
        self.intialize_db(name, docs_loc, embedder)
    
    def initialize_db(self, name: str, docs_loc: str, embedder: OpenAIEmbeddings):
        pass