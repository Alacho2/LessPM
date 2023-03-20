import {useEffect, useState} from "react";

const BASE_URL = "https://localhost:3000/";
const AUTHENTICATED_URL = `${BASE_URL}user/authenticated`;

const checkIsAuthenticated = () => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    request();
  }, []);

  const request = async () => {
    try {
      const authenticated = await fetch(`${BASE_URL}user/authenticated`, {
        method: "GET",
        credentials: "include",
      });
      setIsAuthenticated(authenticated.status === 200);
    } catch (e) { /* Don't do anything */ }
  };

  return isAuthenticated
};

export default checkIsAuthenticated;