import api, { ApiError, HttpResponse } from "../../api";
import { accessTokenAtom, forceLogout, login, logout, refresh } from "../../services/auth";
import { toast } from "../../utils";
import {
  Button,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Heading,
  Input,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalFooter,
  ModalHeader,
  ModalOverlay,
  Stack,
  VStack,
  useDisclosure,
} from "@chakra-ui/react";
import { Turnstile } from "@marsidev/react-turnstile";
import { useAtom } from "jotai";
import React, { useCallback, useEffect } from "react";
import { SubmitHandler, useForm } from "react-hook-form";

type Inputs = {
  username: string;
  password: string;
};

export const UserInfo: React.FC = () => {
  const [,] = useAtom(accessTokenAtom);
  const [refreshToken] = useAtom(accessTokenAtom);

  const {
    data: session,
    isLoading,
    error,
  } = api.useSessionGet({ shouldRetryOnError: false, revalidateOnFocus: false });
  const { data: config } = api.useSessionGetConfig();

  const { isOpen, onOpen, onClose } = useDisclosure();
  const [token, setToken] = React.useState<string | null>(null);

  const {
    register,
    handleSubmit,
    formState: { errors: formErrs, isDirty },
  } = useForm<Inputs>({
    defaultValues: { username: "", password: "" },
  });

  const toastError = useCallback((err: unknown) => {
    toast({
      title: (err as HttpResponse<void, ApiError>).error.message,
      status: "error",
      duration: 5000,
      isClosable: true,
    });
  }, []);
  const onSubmit: SubmitHandler<Inputs> = async (inputs) => {
    try {
      await login(inputs.username, inputs.password, token ?? "");
    } catch (err) {
      toastError(err);
    }
  };
  const onLogout = async () => {
    try {
      await logout();
    } catch (err) {
      toastError(err);
    }
  };

  useEffect(() => {
    void (async () => {
      if (error?.error_code === "INVALID_TOKEN" || error?.error_code === "INVALID_TOKEN_NOT_FIRST_PARTY") {
        if (refreshToken !== null) {
          try {
            await refresh();
          } catch (err) {
            toastError(err);
            await forceLogout();
          }
        } else {
          onOpen();
        }
      }
    })();
  }, [error, toastError, onOpen, refreshToken]);

  if (isLoading) {
    return (
      <Heading as="h2" size="md" display="inline">
        Loading
      </Heading>
    );
  }

  if (error?.error_code === "INVALID_TOKEN") {
    if (refreshToken !== null) {
      return (
        <Heading as="h2" size="md" display="inline">
          Refreshing Token
        </Heading>
      );
    } else {
      return (
        <>
          <Stack direction="row" spacing={4} align="center">
            <Button colorScheme="white" variant="link" onClick={onOpen} size="lg">
              Login
            </Button>
          </Stack>
          <Modal isOpen={isOpen} onClose={onClose} size="xl">
            <ModalOverlay />
            <ModalContent>
              <ModalHeader>Login to ZhuiAni.me</ModalHeader>
              <ModalCloseButton />
              <ModalBody>
                <VStack spacing="2">
                  <FormControl isInvalid={!!formErrs.username}>
                    <FormLabel>Username</FormLabel>
                    <Input {...register("username", { required: true })} />
                    <FormErrorMessage>{formErrs.username?.message}</FormErrorMessage>
                  </FormControl>
                  <FormControl isInvalid={!!formErrs.password}>
                    <FormLabel>Password</FormLabel>
                    <Input type="password" {...register("password", { required: true })} />
                    <FormErrorMessage>{formErrs.password?.message}</FormErrorMessage>
                  </FormControl>
                  {config?.turnstile_site_key && (
                    <Turnstile siteKey={config?.turnstile_site_key} onSuccess={setToken} />
                  )}
                </VStack>
              </ModalBody>

              <ModalFooter>
                <Button onClick={onClose} mr="2">
                  Close
                </Button>
                {/* eslint-disable-next-line @typescript-eslint/no-misused-promises */}
                <Button colorScheme="teal" isDisabled={!isDirty || !token} onClick={handleSubmit(onSubmit)}>
                  Login
                </Button>
              </ModalFooter>
            </ModalContent>
          </Modal>
        </>
      );
    }
  }

  return (
    <Stack direction="row" spacing={4} align="center">
      <Heading as="h2" size="md" display="inline">
        {session?.user.username}
      </Heading>
      {/* eslint-disable-next-line @typescript-eslint/no-misused-promises */}
      <Button colorScheme="white" variant="link" onClick={onLogout} size="lg">
        Logout
      </Button>
    </Stack>
  );
};
