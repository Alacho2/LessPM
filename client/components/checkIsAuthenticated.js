import {useEffect, useState} from "react";

const BASE_URL = "https://localhost:3000/";
const AUTHENTICATED_URL = `${BASE_URL}user/authenticated`;

const checkIsAuthenticated = (section) => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    request();
    // we only do a rerender when the section state changes in main.
  }, [section]);

  const request = async () => {
    try {
      const authenticated = await fetch(`${BASE_URL}user/authenticated`, {
        method: "GET",
        credentials: "include",
      });
      setIsAuthenticated(authenticated.status === 200);
    } catch { /* Don't do anything */ }
  };

  return isAuthenticated
};

export default checkIsAuthenticated;