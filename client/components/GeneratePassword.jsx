import { useEffect, useState } from "react";

// PROPS:
// setPassword: (password) => {}
const GeneratePassword = (props) => {
  const [useSmallCase, setUseSmallCase] = useState(true);
  const [useCapitalCase, setUseCapitalCase] = useState(true);
  const [useNumbers, setUseNumbers] = useState(true);
  const [useSpecialSymbols, setUseSpecialSymbols] = useState(false);
  const [range, setRange] = useState(24);

  // componentDidMount
  useEffect(() => generatePasswordBasedOnLength(24), []);

  useEffect(() => generatePasswordBasedOnLength(range),
    [useSmallCase, useCapitalCase, useNumbers, useSpecialSymbols]);

  // PARAMETERS:
  // len: number
  const generatePasswordBasedOnLength = (len) => {
    const smallCase = "abcdefghijklmnopqrstuvwxyz";
    const largeCase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const numbers = "1234567890";
    const specialSymbols = "!@#$%^&*";

    let alphabet = "";
    if (!useSmallCase && !useCapitalCase && !useNumbers && !useSpecialSymbols) {
      alphabet = smallCase;
    } else {
      if (useSmallCase) {
        alphabet = alphabet + smallCase;
      }

      if (useCapitalCase) {
        alphabet = alphabet + largeCase;
      }

      if (useNumbers) {
        alphabet = alphabet + numbers;
      }

      if (useSpecialSymbols) {
        alphabet = alphabet + specialSymbols;
      }
    }

    const alphabetLength = alphabet.length;

    let generatedPassword = "";
    for (let i = 0; i < len; i++) {
      generatedPassword +=
        alphabet.charAt(Math.floor(Math.random() * alphabetLength));
    }

    props.setPassword(generatedPassword);

  };

  const updatedPassword = ({target}) => {
    setRange(Number(target.value));
    generatePasswordBasedOnLength(Number(target.value));
  };

  return (
    <>
      <h4 className="mt-3">Password Generator Options</h4>
      <input
        type="range"
        min="8"
        max="64"
        onChange={updatedPassword}
        value={range}
      />
      <div className="password-options">
        <div className="option form-check">
          <input
            className="form-check-input"
            type="checkbox"
            onChange={() => setUseSmallCase(!useSmallCase)}
            defaultChecked={true}
            id="small-case" />
          <label
            className="form-check-label"
            htmlFor="small-case">
            Small Characters
          </label>
        </div>
        <div className="option form-check">
          <input
            className="form-check-input"
            type="checkbox"
            onChange={() => setUseCapitalCase(!useCapitalCase)}
            defaultChecked={true}
            id="capital-case"/>
          <label
            className="form-check-label"
            htmlFor="capital-case">
            Capital Characters
          </label>
        </div>
        <div className="option form-check">
          <input
            className="form-check-input"
            type="checkbox"
            onChange={({target}) => setUseNumbers(target.value)}
            defaultChecked={true}
            id="numbers-case"/>
          <label
            className="form-check-label"
            htmlFor="numbers-case">
            Numbered Characters
          </label>
        </div>
        <div className="option form-check">
          <input
            className="form-check-input"
            type="checkbox"
            onChange={({target}) => setUseSpecialSymbols(target.value)}
            defaultChecked={false}
            id="special-characters-case"/>
          <label
            className="form-check-label"
            htmlFor="special-characters-case">
            Special Characters
          </label>
        </div>
      </div>
    </>
  )
};

export default GeneratePassword;
