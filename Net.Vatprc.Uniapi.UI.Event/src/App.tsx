import { UserInfo } from "./components/user/UserInfo";
import { Item } from "./pages/Item";
import { ItemsList } from "./pages/ItemsList";
import { LoginPage } from "./pages/Login";
import { PikPakTasksList } from "./pages/PikPakTasksList";
import { TorrentsList } from "./pages/Torrents";
import { hasAuthenticatedAtom, refresh, refreshTokenAtom } from "./services/auth";
import { promiseWithToast } from "./utils";
import { Divider, Heading, Stack, chakra } from "@chakra-ui/react";
import { useAtom } from "jotai";
import React, { useEffect } from "react";
import { Link, Route, Routes } from "react-router-dom";

const App: React.FunctionComponent = () => {
  const [hasAuthenticated] = useAtom(hasAuthenticatedAtom);
  const [refreshToken] = useAtom(refreshTokenAtom);

  useEffect(() => {
    if (!hasAuthenticated && refreshToken) {
      promiseWithToast(refresh());
    }
  }, [refreshToken, hasAuthenticated]);

  if (!hasAuthenticated && !refreshToken) {
    return <LoginPage />;
  }

  return (
    <Stack direction={["column", "row"]}>
      <Stack
        bg="green.500"
        minH={["auto", "100vh"]}
        minW={["100vw", "auto"]}
        paddingX="6"
        paddingY="4"
        color="white"
        spacing="4"
      >
        <Heading color="green.50" as="h1" size="2xl">
          <chakra.span color="green.100" fontWeight="light">
            Zhui
          </chakra.span>
          Ani<chakra.span fontWeight="light">.</chakra.span>me
        </Heading>
        <Heading as="h2" size="lg">
          <Link to="/">Animations</Link>
        </Heading>
        <Heading as="h2" size="lg">
          <Link to="/torrents">Torrents</Link>
        </Heading>
        <Heading as="h2" size="lg">
          <Link to="/pikpak/tasks">PikPak</Link>
        </Heading>
        <Divider />
        <UserInfo />
      </Stack>
      <Stack padding="4" bg="gray.50" spacing="4" flexGrow="1">
        <Routes>
          <Route path="/" element={<ItemsList />}></Route>
          <Route path="/animes">
            <Route path=":animeId" element={<Item />} />
          </Route>
          <Route path="/torrents" element={<TorrentsList />}></Route>
          <Route path="/pikpak/tasks" element={<PikPakTasksList />}></Route>
        </Routes>
      </Stack>
    </Stack>
  );
};

export default App;
