import logo from "../assets/standard.svg";
import { CreateEvent } from "@/components/event-create";
import { Login } from "@/components/login";
import { DevLogin } from "@/components/login-dev";
import { ANONYMOUS_CID, logout, useUser } from "@/services/auth";
import { promiseWithToast } from "@/utils";
import { Button, Container, Group, Image, Stack } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import { Link, Outlet, createRootRoute } from "@tanstack/react-router";

const Logout = () => {
  const user = useUser();

  const onClick = () => {
    promiseWithToast(logout());
  };

  if (user.cid === ANONYMOUS_CID) return null;
  return (
    <>
      <Button variant="transparent">{user.cid}</Button>
      <Button variant="outline" onClick={onClick}>
        Logout
      </Button>
    </>
  );
};

export const Route = createRootRoute({
  component: () => (
    <Container mb="lg">
      <Stack mt={16}>
        <Group justify="space-between">
          <Group>
            <Image src={logo} alt="logo" h={32} />
            <Link
              to="/"
              activeOptions={{ exact: true }}
              children={(state) => <Button variant={state.isActive ? "light" : "subtle"}>Events</Button>}
            />
          </Group>
          <Group>
            <CreateEvent />
            <DevLogin />
            <Login />
            <Logout />
          </Group>
        </Group>
        <Outlet />
      </Stack>
      <Notifications position="top-center" />
    </Container>
  ),
});
