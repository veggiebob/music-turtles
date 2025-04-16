import React from "react";
import { Terminal } from "./protocol";



interface TerminalProps {
  terminal: Terminal;
}

const TerminalComp: React.FC<TerminalProps> = ({ terminal }) => {
  if (terminal.type === "Music") {
    return (
      <div>
        <h3>Music</h3>
        <p>Duration: {terminal.duration}</p>
        <p>Note: {terminal.note}</p>
      </div>
    );
  }

  if (terminal.type === "Meta") {
    return (
      <div>
        <h3>Meta</h3>
        <p>Control: {terminal.control}</p>
      </div>
    );
  }

  return null;
};

export default TerminalComp;