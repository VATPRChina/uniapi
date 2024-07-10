import { useUser } from "@/services/auth";
import { Button } from "@mantine/core";

export const Login = () => {
  const user = useUser();

  const url = new URL("/auth/authorize", import.meta.env.VITE_VATPRC_UNIAPI_ENDPOINT);
  url.searchParams.set("client_id", "01J2ED6BDX9J9BTYS2RVW83ME7");
  url.searchParams.set("redirect_uri", "http://localhost:3000/auth/callback");
  url.searchParams.set("response_type", "code");

  if (user) return null;
  return (
    <Button variant="outline" component="a" href={url.toString()}>
      VATSIM Connect
    </Button>
  );
};
