import logo from "../assets/standard.svg";
import { CreateEvent } from "@/components/create-event";
import { DevLogin } from "@/components/dev-login";
import { Button, Container, Group, Image, Stack } from "@mantine/core";
import { Link, Outlet, createRootRoute } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";

export const Route = createRootRoute({
  component: () => (
    <Container>
      <Stack mt={16}>
        <Group justify="space-between">
          <Group>
            <Image src={logo} alt="logo" h={32} />
            <Link to="/">
              <Button variant="light">Home</Button>
            </Link>
            <Link to="/about">
              <Button variant="subtle">About</Button>
            </Link>
          </Group>
          <Group>
            <CreateEvent />
            <DevLogin />
          </Group>
        </Group>
        <Outlet />
      </Stack>
      <TanStackRouterDevtools />
    </Container>
  ),
});
