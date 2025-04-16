import { useEffect, useState } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'
import { Grammar, MusicString } from './protocol';

function getProductions(grammar: Grammar, name: string): MusicString[] {
  const productions = grammar.productions.filter((p) => {
    if (p[0].Custom === name) {
      return true;
    }
    return false;
  }).map((p) => p[1]);
  return productions;
}

function App() {
  const [grammar, setGrammar] = useState(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const res = await fetch('http://localhost:8000/grammar/grm1.grm');
        const json = await res.json();
        console.log(json);
        setGrammar(json.message); // assuming your JSON has a `message` field
      } catch (error) {
        console.error('Error fetching data:', error);
      }
    };

    fetchData();
  }, []); // ← empty array = run once when mounted
  return (
    <>
      
    </>
  )
}

export default App
