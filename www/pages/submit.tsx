import type { NextPage } from "next";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Button from "@mui/material/Button";
import Typography from "@mui/material/Typography";
import Container from "@mui/material/Container";
import Alert from "@mui/material/Alert";
import { zodResolver } from "@hookform/resolvers/zod";
import { FieldValues, useForm, UseFormSetError } from "react-hook-form";
import { z } from "zod";
import { useCallback } from "react";
import { useRouter } from "next/router";

const schema = z.object({
  args: z.string(),
  target: z.string(),
});

type Schema = z.infer<typeof schema>;

const Page: NextPage<{}> = ({}) => {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
    setError,
    // clearErrors,
  } = useForm({
    mode: "onSubmit",
    reValidateMode: "onSubmit",
    resolver: zodResolver(schema),
  });

  const router = useRouter();

  const hasError = Object.keys(errors).length > 0;

  const handle = useCallback(
    (data: Schema) => {
      console.log(data);
      router.push({ pathname: "/" });
    },
    [router]
  );

  const argsHelper = (errors.args?.message as any) || "-a DP,Refine とか書く";
  const targetHelper =
    (errors.target?.message as any) || "1-25 とか 1,2,3 とか実行対象を書く";

  return (
    <Stack
      component="form"
      noValidate
      onSubmit={handleSubmit(handle as any)}
      spacing={2}
    >
      <Typography component="h1" variant="h5">
        ソルバー実行
      </Typography>

      <TextField
        margin="normal"
        required
        fullWidth
        label="引数"
        autoComplete="args"
        {...register("args")}
        helperText={argsHelper}
        error={"args" in errors}
      />

      <TextField
        margin="normal"
        required
        fullWidth
        label="対象"
        autoComplete="target"
        {...register("target")}
        helperText={targetHelper}
        error={"target" in errors}
      />

      {hasError && (
        <Alert sx={{ width: "100%" }} severity="error">
          入力内容が間違ってるよ
        </Alert>
      )}

      <Button
        variant="contained"
        type="submit"
        fullWidth
        disabled={isSubmitting}
      >
        実行
      </Button>
    </Stack>
  );
};

export default Page;
