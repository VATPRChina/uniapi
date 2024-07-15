import { ANONYMOUS_CID, useUser } from "@/services/auth";
import { Button } from "@mantine/core";

export const Login = () => {
  const user = useUser();

  const url = new URL("/auth/authorize", import.meta.env.VITE_API_ENDPOINT);
  url.searchParams.set("client_id", import.meta.env.VITE_API_CLIENT_ID);
  url.searchParams.set("redirect_uri", import.meta.env.VITE_API_REDIRECT_URI);
  url.searchParams.set("response_type", "code");

  if (user.cid !== ANONYMOUS_CID) return null;
  return (
    <Button variant="outline" component="a" href={url.toString()}>
      Login
    </Button>
  );
};
