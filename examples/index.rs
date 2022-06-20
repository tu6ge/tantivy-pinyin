use tantivy::collector::{Count, TopDocs};
use tantivy::query::TermQuery;
use tantivy::schema::*;
use tantivy::{doc, Index, ReloadPolicy};
use tantivy::tokenizer::{PreTokenizedString, Token, Tokenizer};
use tempfile::TempDir;

use tantivy_pinyin::PinyinTokenizer;

fn pre_tokenize_text(text: &str) -> Vec<Token> {
  let mut token_stream = PinyinTokenizer.token_stream(text);
  let mut tokens = vec![];
  while token_stream.advance() {
      tokens.push(token_stream.token().clone());
  }
  tokens
}


pub fn main() -> tantivy::Result<()> {
  let index_path = TempDir::new()?;

  let mut schema_builder = Schema::builder();

  schema_builder.add_text_field("title", TEXT | STORED);
  schema_builder.add_text_field("body", TEXT);

  let schema = schema_builder.build();

  let index = Index::create_in_dir(&index_path, schema.clone())?;

  let mut index_writer = index.writer(50_000_000)?;

  // We can create a document manually, by setting the fields
  // one by one in a Document object.
  let title = schema.get_field("title").unwrap();
  let body = schema.get_field("body").unwrap();

  let title_text = "大多数知识，不需要我们记住";
  let body_text = "大多数知识，只需要认知即可";

  // Content of our first document
  // We create `PreTokenizedString` which contains original text and vector of tokens
  let title_tok = PreTokenizedString {
      text: String::from(title_text),
      tokens: pre_tokenize_text(title_text),
  };

  println!(
      "Original text: \"{}\" and tokens: {:?}",
      title_tok.text, title_tok.tokens
  );

  let body_tok = PreTokenizedString {
      text: String::from(body_text),
      tokens: pre_tokenize_text(body_text),
  };

  // Now lets create a document and add our `PreTokenizedString`
  let old_man_doc = doc!(title => title_tok, body => body_tok);

  // ... now let's just add it to the IndexWriter
  index_writer.add_document(old_man_doc)?;

  // Let's commit changes
  index_writer.commit()?;

  // ... and now is the time to query our index

  let reader = index
      .reader_builder()
      .reload_policy(ReloadPolicy::OnCommit)
      .try_into()?;

  let searcher = reader.searcher();

  // We want to get documents with token "Man", we will use TermQuery to do it
  // Using PreTokenizedString means the tokens are stored as is avoiding stemming
  // and lowercasing, which preserves full words in their original form
  let query = TermQuery::new(
      //Term::from_field_text(title, "liu"),
      Term::from_field_text(body, "xin"),
      IndexRecordOption::Basic,
  );

  let (top_docs, count) = searcher.search(&query, &(TopDocs::with_limit(2), Count))?;

  println!("Found {} documents", count);

  // Now let's print out the results.
  // Note that the tokens are not stored along with the original text
  // in the document store
  for (_score, doc_address) in top_docs {
      let retrieved_doc = searcher.doc(doc_address)?;
      println!("Document: {}", schema.to_json(&retrieved_doc));
  }

  Ok(())
}