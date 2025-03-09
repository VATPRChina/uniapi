import logo from "../assets/standard.svg";
import { Button, Container, Group, Image, Stack } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import { Link, Outlet, createRootRoute } from "@tanstack/react-router";

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
              children={(state) => <Button variant={state.isActive ? "light" : "subtle"}>Flights</Button>}
            />
          </Group>
          <Group></Group>
        </Group>
        <Outlet />
      </Stack>
      <Notifications position="top-center" />
    </Container>
  ),
});
