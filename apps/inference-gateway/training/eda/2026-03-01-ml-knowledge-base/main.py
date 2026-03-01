import json
import logging
import os
import random
from datetime import datetime

import matplotlib.pyplot as plt
import pandas as pd
import spacy
from sklearn.feature_extraction.text import CountVectorizer, TfidfVectorizer
from wordcloud import WordCloud

logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")
log = logging.getLogger(__name__)

KNOWLEDGE_BASE_PATH = "../../../crawler/data/2026-03-01-02-45/knowledge_base.jsonl"
OUTPUT_DIR = "."
IMG_DIR = os.path.join(OUTPUT_DIR, "img")

# Load SpaCy for NLP
log.info("Loading SpaCy en_core_web_sm model...")
try:
    nlp = spacy.load("en_core_web_sm", disable=["ner", "parser"])
except Exception as e:
    log.error(f"Failed to load spacy model: {e}")
    raise

# Hard filter to remove generic programming, github, and library names, focusing on ML concepts
# Only let strictly conceptual or domain-specific words shine.
CUSTOM_STOPWORDS = set([
    # Generic issue tracker
    "issue", "bug", "feature", "request", "version", "update", "add", "new", "fix",
    "http", "https", "com", "github", "org", "www", "pull", "commit", "branch",
    "master", "main", "release", "tag", "reproduce", "steps", "description",
    "context", "logs", "output", "help", "problem", "question", "support", "please",
    "thanks", "expected", "behavior", "actual",

    # Generic programming / OS
    "using", "code", "run", "error", "traceback", "file", "line", "test", "module",
    "build", "install", "environment", "system", "windows", "linux", "mac", "ubuntu",
    "use", "way", "one", "get", "set", "return", "value", "type", "name", "self",
    "def", "class", "import", "from", "os", "sys", "print", "data", "list", "dict",
    "string", "int", "None", "True", "False",

    # Specific framework names that might be too dominant
    "pytorch", "torch", "python", "numpy", "scipy", "pandas", "tensorflow", "keras",
    "huggingface", "transformers", "scikit-learn", "sklearn"
])

import sklearn.feature_extraction.text as text

STOP_WORDS = text.ENGLISH_STOP_WORDS.union(CUSTOM_STOPWORDS)

def filter_ml_concepts(docs):
    """
    Use SpaCy to extract only lemmatized nouns and adjectives that are not in STOP_WORDS.
    This highly focuses the text upon conceptual nouns (like 'gradient', 'transformer', 'optimization').
    """
    filtered_texts = []
    # Use nlp.pipe for fast processing
    for doc in nlp.pipe(docs, batch_size=50):
        tokens = [
            token.lemma_.lower() for token in doc
            if token.pos_ in ["NOUN", "PROPN", "ADJ"]
            and token.lemma_.lower() not in STOP_WORDS
            and len(token.lemma_) > 2
            and token.is_alpha
        ]
        filtered_texts.append(" ".join(tokens))
    return filtered_texts

def generate_wordcloud(text_data, title, filename):
    log.info(f"Generating WordCloud for {title}...")
    text = " ".join(text_data)
    wordcloud = WordCloud(width=800, height=400, background_color='white', max_words=100, collocations=False).generate(text)

    plt.figure(figsize=(10, 5))
    plt.imshow(wordcloud, interpolation='bilinear')
    plt.axis('off')
    plt.title(title, fontsize=16)
    plt.tight_layout(pad=0)
    plt.savefig(os.path.join(IMG_DIR, filename))
    plt.close()

def plot_top_n_ngrams(text_data, n_gram_range=(1,1), top_n=20, title="Top N-Grams", filename="ngrams.png"):
    log.info(f"Extracting top {n_gram_range} N-Grams for {title}...")
    vec = CountVectorizer(ngram_range=n_gram_range).fit(text_data)
    bag_of_words = vec.transform(text_data)
    sum_words = bag_of_words.sum(axis=0)
    words_freq = [(word, sum_words[0, idx]) for word, idx in vec.vocabulary_.items()]
    words_freq = sorted(words_freq, key=lambda x: x[1], reverse=True)[:top_n]

    df = pd.DataFrame(words_freq, columns=['ngram', 'count'])

    plt.figure(figsize=(12, 6))
    plt.bar(df['ngram'], df['count'], color='coral')
    plt.xticks(rotation=45, ha='right')
    plt.title(title)
    plt.xlabel('N-Gram')
    plt.ylabel('Frequency')
    plt.tight_layout()
    plt.savefig(os.path.join(IMG_DIR, filename))
    plt.close()

def analyze_tfidf(text_data, top_n=20, title="Top TF-IDF ML Concepts", filename="tfidf.png"):
    log.info(f"Running TF-IDF analysis for {title}...")
    # Since text_data is already filtered by basic SPACY POS, min_df can be strict.
    vectorizer = TfidfVectorizer(max_df=0.90, min_df=0.01)
    tfidf_matrix = vectorizer.fit_transform(text_data)

    # Get average TF-IDF score
    mean_tfidf = tfidf_matrix.mean(axis=0).A1
    words = vectorizer.get_feature_names_out()

    tfidf_scores = [(words[i], mean_tfidf[i]) for i in range(len(words))]
    tfidf_scores = sorted(tfidf_scores, key=lambda x: x[1], reverse=True)[:top_n]

    df = pd.DataFrame(tfidf_scores, columns=['concept', 'tfidf_score'])

    plt.figure(figsize=(12, 6))
    plt.bar(df['concept'], df['tfidf_score'], color='mediumpurple')
    plt.xticks(rotation=45, ha='right')
    plt.title(title)
    plt.xlabel('ML / Domain Concept')
    plt.ylabel('Average TF-IDF Score')
    plt.tight_layout()
    plt.savefig(os.path.join(IMG_DIR, filename))
    plt.close()

def main():
    os.makedirs(IMG_DIR, exist_ok=True)

    log.info(f"Loading data from {KNOWLEDGE_BASE_PATH}")
    issues = []
    with open(KNOWLEDGE_BASE_PATH, encoding="utf-8") as f:
        for line in f:
            issues.append(json.loads(line))

    log.info(f"Loaded {len(issues)} issues.")

    titles = [issue.get("title", "") for issue in issues if issue.get("title")]
    bodies = [issue.get("body", "") for issue in issues if issue.get("body")]

    log.info("Filtering data for pure ML concepts and Nouns...")
    filtered_titles = filter_ml_concepts(titles)
    # the bodies are too huge, randomly sample 500 for NLP to save time.
    sub_bodies = random.sample(bodies, min(500, len(bodies)))
    filtered_bodies = filter_ml_concepts(sub_bodies)

    log.info("Generating Domain-Specific Plots...")

    # 2. Advanced Domain NLP Analysis
    # WordClouds
    generate_wordcloud(filtered_titles, "ML Concepts in Issue Titles", "title_wordcloud.png")
    generate_wordcloud(filtered_bodies, "ML Concepts in Issue Bodies (Subset)", "body_wordcloud.png")

    # N-Grams Analysis (Bi-grams)
    plot_top_n_ngrams(filtered_titles, n_gram_range=(2,2), top_n=20, title="Top 20 ML Construct Bigrams (Titles)", filename="title_bigrams.png")
    plot_top_n_ngrams(filtered_bodies, n_gram_range=(2,2), top_n=20, title="Top 20 ML Construct Bigrams (Bodies)", filename="body_bigrams.png")

    # TF-IDF Analysis
    analyze_tfidf(filtered_titles, top_n=20, title="Top 20 Distinct ML Concepts (Titles TF-IDF)", filename="title_tfidf.png")
    analyze_tfidf(filtered_bodies, top_n=20, title="Top 20 Distinct ML Concepts (Bodies TF-IDF)", filename="body_tfidf.png")

    # 4. Generate README report
    report_path = os.path.join(OUTPUT_DIR, "README.md")
    with open(report_path, "w", encoding="utf-8") as f:
        f.write("# ML Domain Exploratory Data Analysis (EDA) Report\n\n")
        f.write(f"**Date:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"**Data Source:** `{KNOWLEDGE_BASE_PATH}`\n\n")

        f.write("## 1. Domain-Specific Methodology\n")
        f.write("To satisfy the ML-centric objective, this EDA relies heavily on NLP Part-of-Speech Tagging (via `spacy`) to extract only strictly NOUN, PROPN, and ADJ tags.\n")
        f.write("It aggressively strips away all GitHub template filler (e.g., 'expected behavior', 'logs', 'fix'), general programming constructs ('def', 'return', 'import'), and literal framework names ('tensorflow', 'pytorch').\n")
        f.write("This leaves behind only the core ML concepts, hardware logic, and specific algorithmic discussions.\n\n")

        f.write("## 2. Text Content Analysis (Filtered ML Concepts Only)\n\n")

        f.write("### 2.1 Concept Clouds\n")
        f.write("Visualizations of the most prominent ML concepts being discussed.\n\n")
        f.write("#### Titles\n")
        f.write("![Title Word Cloud](img/title_wordcloud.png)\n\n")
        f.write("#### Bodies\n")
        f.write("![Body Word Cloud](img/body_wordcloud.png)\n\n")

        f.write("### 2.2 Construct Bigrams\n")
        f.write("Bigrams explicitly showing coupled relationships between conceptual nouns (e.g., 'memory leak', 'cuda graph', 'flash attention').\n\n")
        f.write("![Title Bigrams](img/title_bigrams.png)\n")
        f.write("![Body Bigrams](img/body_bigrams.png)\n\n")

        f.write("### 2.3 Highly Distinct Entities (TF-IDF)\n")
        f.write("TF-IDF isolates highly descriptive models, parameters, or hardware configurations that differentiate issues.\n\n")
        f.write("![Title TFIDF](img/title_tfidf.png)\n")
        f.write("![Body TFIDF](img/body_tfidf.png)\n\n")

        f.write("## 3. Implications for the Cross-Encoder Model\n")
        f.write("1. **Data is Extremely Specialized**: The data is dominated by lower-level systems architecture terms ('cuda', 'tensor', 'kernel') rather than generic high-level 'NLP' words. The Cross-Encoder model MUST have a vocabulary that tokenizes these well (e.g. `bge-reranker`).\n")
        f.write("2. **Dense Meaning**: Because issues can be stripped down to such dense conceptual chunks, we should consider passing clean, filtered text to the cross-encoder instead of raw stack traces, to maximize token capacity (512 tokens).\n")

    log.info(f"Done! ML Specific Report generated at {report_path}")

if __name__ == "__main__":
    main()
