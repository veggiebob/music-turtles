import React from 'react';
import { TracedString } from './protocol';


type TracedStringProps = {
  tracedString: TracedString;
};

const TracedStringComponent: React.FC<TracedStringProps> = ({ tracedString }) => {
  return (
    <div>
      <h3>Original:</h3>
      <p>{tracedString.original}</p>
      <h3>Productions:</h3>
      <ul>
        {Array.from(tracedString.productions.entries()).map(([key, [production, subTracedString]]) => (
          <li key={key}>
            <strong>Step {key}:</strong> {production}
            <TracedStringComponent tracedString={subTracedString} />
          </li>
        ))}
      </ul>
    </div>
  );
};

export default TracedStringComponent;