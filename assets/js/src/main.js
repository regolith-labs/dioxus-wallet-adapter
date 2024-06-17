import ReactDOM from 'react-dom/client';

function App() {
  return <h1>Plssssss</h1>;
}

function renderr() {
  const container = document.getElementById('plsss');
  const root = ReactDOM.createRoot(container);
  root.render(<App />);
}

window.PlsRender = renderr;
