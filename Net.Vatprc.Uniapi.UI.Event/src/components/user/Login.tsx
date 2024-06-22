import api, { ApiError, HttpResponse } from "../../api";
import { login } from "../../services/auth";
import { promiseWithLog } from "../../utils";
import { ErrorTip } from "../utils/ErrorTip";
import { Button, FormControl, FormErrorMessage, FormLabel, Input, VStack } from "@chakra-ui/react";
import { Turnstile, TurnstileInstance } from "@marsidev/react-turnstile";
import React, { MouseEventHandler } from "react";
import { useForm } from "react-hook-form";

type Inputs = {
  username: string;
  password: string;
};

export const Login: React.FC = () => {
  const { data: config } = api.useSessionGetConfig();

  const [token, setToken] = React.useState<string | null>(null);
  const [loading, setLoading] = React.useState(false);
  const turnstile = React.useRef<TurnstileInstance>();

  const {
    register,
    handleSubmit,
    formState: { errors: formErrs, isDirty },
    setError,
  } = useForm<Inputs>({
    defaultValues: { username: "", password: "" },
  });

  const onSubmit: MouseEventHandler<HTMLButtonElement> = (e) =>
    promiseWithLog(
      handleSubmit(async (inputs) => {
        try {
          setLoading(true);
          await login(inputs.username, inputs.password, token ?? "");
        } catch (err) {
          setError("root", { message: (err as HttpResponse<void, ApiError>).error.message });
          turnstile.current?.reset();
        }
        setLoading(false);
      })(e),
    );

  return (
    <form>
      <VStack spacing="2">
        <ErrorTip error={formErrs.root} />
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
          <Turnstile ref={turnstile} siteKey={config?.turnstile_site_key} onSuccess={setToken} />
        )}
        <Button colorScheme="teal" isDisabled={!isDirty || !token} onClick={onSubmit} type="submit" isLoading={loading}>
          Login
        </Button>
      </VStack>
    </form>
  );
};
