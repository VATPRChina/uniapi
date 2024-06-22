import { Alert, AlertDescription, AlertIcon } from "@chakra-ui/react";
import React from "react";

export const ErrorTip = ({ error }: { error?: { message?: string } }) =>
  error?.message && (
    <Alert status="error">
      <AlertIcon />
      <AlertDescription>{error.message}</AlertDescription>
    </Alert>
  );
