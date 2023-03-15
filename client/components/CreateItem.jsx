
  // This component should create the VaultItem.
  // It should contain the username
  // It should contain the password
  // It should not generate an ID because that's Mongo's job.
  // It should OFFER to allow the user to write their own password
  // Or generate a strong password of sorts.
  // Finally, to store the password, it should prompt for the auth
  // And then ship the info to the server on success
  // Server should AGAIN validate the token that the user is authenticated.

  // All the information is sent with the auth finish request.
  // That should deal with everything that needs to be in terms of the
  // server accepting info.
import { useEffect, useState } from "react";

const CreateItem = () => {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [checked, setChecked] = useState(true);

  // I want the input and password to either be a grid or a flexbox

  console.log(checked);
  return (
    <div className="mt-4 bg-light border rounded create-item">
      <div className="mx-3 my-3">
        <div className="controller mb-3">
            <div className="havard-class">
          <div className="input-group">
          <div className="input-group-prepend">
            <span className="input-group-text" id="basic-addon1">@</span>
          </div>
          <input
            type="text"
            className="form-control"
            placeholder="Username"
            aria-label="Username"
            onChange={({target}) => setUsername(target.value)}
            aria-describedby="basic-addon1" value={username} />
          </div>
          </div>

            <div className="havard-class password">
          <div className="input-group">

          <div className="input-group-prepend">
            <div className="input-group-text">
              <input
                className="checkbox-controlled-height"
                type="checkbox"
                defaultChecked={true}
                onChange={({target}) => setChecked(target.checked)}
                aria-label="Checkbox for following text input" />
            </div>
          </div>
          <input
            type="text" // This depends on the checkbox
            className="form-control"
            placeholder="P4sSw0rd"
            disabled={checked}
            aria-label="Text input with checkbox" />
            </div>
          </div>
        </div>

      </div>
    </div>
  )
};

export default CreateItem;
