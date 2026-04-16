import { render } from 'preact';

import App from './App';
import 'katex/dist/katex.min.css';
import './styles.css';

const root = document.getElementById('app');

if (!root) {
  throw new Error('Missing #app root');
}

render(<App />, root);
