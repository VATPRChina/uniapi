import logo from "../assets/standard.svg";
import { Button, Container, Group, Image, Stack } from "@mantine/core";
import { Link, Outlet, createRootRoute } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";

export const Route = createRootRoute({
  component: () => (
    <Container>
      <Stack mt={16}>
        <Group>
          <Image src={logo} alt="logo" h={32} />
          <Link to="/">
            <Button variant="light">Home</Button>
          </Link>
          <Link to="/about">
            <Button variant="subtle">About</Button>
          </Link>
        </Group>
        <Outlet />
      </Stack>
      <TanStackRouterDevtools />
    </Container>
  ),
});
