import { Login } from "../components/user/Login";
import { Card, CardBody, CardHeader, Container, Flex, Heading } from "@chakra-ui/react";
import { chakra } from "@chakra-ui/system";
import React from "react";

export const LoginPage: React.FC = () => (
  <Flex minH="100vh" direction="column" justifyContent="center" backgroundColor="gray.50">
    <Container>
      <Card>
        <CardHeader>
          <Heading color="green.900" as="h1" size="2xl">
            <chakra.span color="green.700" fontWeight="light">
              Zhui
            </chakra.span>
            Ani<chakra.span fontWeight="light">.</chakra.span>me
          </Heading>
        </CardHeader>
        <CardBody>
          <Login />
        </CardBody>
      </Card>
    </Container>
  </Flex>
);
