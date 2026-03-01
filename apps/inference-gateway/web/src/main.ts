import './style.css'

interface RankedIssueDto {
  repo_name: string;
  number: number;
  title: string;
  html_url: string;
  score: number;
}

interface RecommendStreamMeta {
  query: string;
  related_issues: RankedIssueDto[];
}

const API_BASE_URL = 'http://127.0.0.1:8800/api';

document.addEventListener('DOMContentLoaded', () => {
  const form = document.getElementById('recommend-form') as HTMLFormElement;
  const input = document.getElementById('query-input') as HTMLInputElement;
  const submitBtn = document.getElementById('submit-btn') as HTMLButtonElement;

  const resultsSection = document.getElementById('results-section') as HTMLElement;
  const loadingIndicator = document.getElementById('loading-indicator') as HTMLElement;
  const contentDisplay = document.getElementById('content-display') as HTMLElement;
  const errorState = document.getElementById('error-state') as HTMLElement;

  const adviceContent = document.getElementById('llm-advice-content') as HTMLElement;
  const issuesList = document.getElementById('related-issues-list') as HTMLUListElement;

  let activeSource: EventSource | null = null;

  form.addEventListener('submit', async (e) => {
    e.preventDefault();

    const query = input.value.trim();
    if (!query) return;

    if (activeSource) {
      activeSource.close();
      activeSource = null;
    }

    // Reset UI state
    resultsSection.classList.remove('hidden');
    loadingIndicator.classList.remove('hidden');
    contentDisplay.classList.add('hidden');
    errorState.classList.add('hidden');
    submitBtn.disabled = true;

    adviceContent.textContent = '';
    issuesList.innerHTML = '';

    try {
      const streamUrl = `${API_BASE_URL}/recommend/stream?query=${encodeURIComponent(query)}`;
      const source = new EventSource(streamUrl);
      activeSource = source;

      let gotMeta = false;

      source.addEventListener('meta', (event) => {
        const e = event as MessageEvent;
        const meta: RecommendStreamMeta = JSON.parse(e.data);
        gotMeta = true;

        // Populate Issues List
        issuesList.innerHTML = '';
        if (!meta.related_issues || meta.related_issues.length === 0) {
          const li = document.createElement('li');
          li.className = 'text-muted';
          li.textContent = 'No related issues found.';
          issuesList.appendChild(li);
        } else {
          meta.related_issues.forEach(issue => {
            const li = document.createElement('li');

            const a = document.createElement('a');
            a.href = issue.html_url;
            a.target = '_blank';
            a.rel = 'noreferrer';
            a.className = 'issue-title';
            a.textContent = issue.title;

            const metaDiv = document.createElement('div');
            metaDiv.className = 'issue-meta';

            const repoSpan = document.createElement('span');
            repoSpan.textContent = `${issue.repo_name}#${issue.number}`;

            const scoreSpan = document.createElement('span');
            scoreSpan.className = 'score-badge';
            scoreSpan.textContent = `Score: ${issue.score.toFixed(3)}`;

            metaDiv.appendChild(repoSpan);
            metaDiv.appendChild(scoreSpan);

            li.appendChild(a);
            li.appendChild(metaDiv);
            issuesList.appendChild(li);
          });
        }

        // Show content as soon as meta arrives
        loadingIndicator.classList.add('hidden');
        contentDisplay.classList.remove('hidden');
      });

      source.addEventListener('delta', (event) => {
        const e = event as MessageEvent;
        adviceContent.textContent = (adviceContent.textContent ?? '') + e.data;
      });

      source.addEventListener('server_error', (event) => {
        const e = event as MessageEvent;
        console.error('Server streaming error:', e.data);
        errorState.textContent = `Failed to stream recommendation: ${e.data}`;
        errorState.classList.remove('hidden');
        loadingIndicator.classList.add('hidden');
        submitBtn.disabled = false;
        source.close();
        activeSource = null;
      });

      source.addEventListener('done', () => {
        submitBtn.disabled = false;
        source.close();
        activeSource = null;
      });

      source.onerror = () => {
        // Connection errors (including CORS / server down)
        if (!gotMeta) {
          errorState.textContent = 'Failed to stream recommendation. Please make sure the API server is running.';
          errorState.classList.remove('hidden');
          loadingIndicator.classList.add('hidden');
        }
        submitBtn.disabled = false;
        source.close();
        activeSource = null;
      };

    } catch (error) {
      console.error('Failed to fetch recommendation:', error);
      errorState.textContent = 'Failed to generate recommendation. Please make sure the API server is running.';
      errorState.classList.remove('hidden');
      loadingIndicator.classList.add('hidden');
    } finally {
      // submitBtn is re-enabled by stream completion
    }
  });
});
