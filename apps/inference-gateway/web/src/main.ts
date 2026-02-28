import './style.css'

interface RankedIssueDto {
  id: string;
  title: string;
  score: number;
}

interface RecommendResponse {
  query: string;
  llm_advice: string;
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

  form.addEventListener('submit', async (e) => {
    e.preventDefault();

    const query = input.value.trim();
    if (!query) return;

    // Reset UI state
    resultsSection.classList.remove('hidden');
    loadingIndicator.classList.remove('hidden');
    contentDisplay.classList.add('hidden');
    errorState.classList.add('hidden');
    submitBtn.disabled = true;

    try {
      const response = await fetch(`${API_BASE_URL}/recommend?query=${encodeURIComponent(query)}`);

      if (!response.ok) {
        throw new Error(`Server returned ${response.status}`);
      }

      const data: RecommendResponse = await response.json();

      // Populate Advice
      adviceContent.textContent = data.llm_advice;

      // Populate Issues List
      issuesList.innerHTML = '';
      if (data.related_issues.length === 0) {
        issuesList.innerHTML = '<li class="text-muted">No related issues found.</li>';
      } else {
        data.related_issues.forEach(issue => {
          const li = document.createElement('li');
          li.innerHTML = `
            <span class="issue-title">${escapeHtml(issue.title)}</span>
            <div class="issue-meta">
              <span>ID: ${escapeHtml(issue.id)}</span>
              <span class="score-badge">Score: ${issue.score.toFixed(3)}</span>
            </div>
          `;
          issuesList.appendChild(li);
        });
      }

      // Show content
      loadingIndicator.classList.add('hidden');
      contentDisplay.classList.remove('hidden');

    } catch (error) {
      console.error('Failed to fetch recommendation:', error);
      errorState.textContent = 'Failed to generate recommendation. Please make sure the API server is running.';
      errorState.classList.remove('hidden');
      loadingIndicator.classList.add('hidden');
    } finally {
      submitBtn.disabled = false;
    }
  });
});

// Basic HTML escaper to prevent XSS
function escapeHtml(unsafe: string): string {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
