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

export const schema = z.object({
  args: z
    .string()
    .min(
      1,
      'パラメーターを入力して下さい (シェルを経由しないため " や * は期待通りに動作しないことがあります)'
    )
    .regex(
      /^[^"';*]+$/,
      "期待通りに動作しないと予想される文字列が含まれています"
    ),
  target: z
    .string()
    .regex(
      /^([1-9][0-9]*|[1-9][0-9]*-[1-9][0-9]*)(,([1-9][0-9]*|[1-9][0-9]*-[1-9][0-9]*))*$/,
      "問題番号の指定が間違っています"
    ),
});

export type Schema = z.infer<typeof schema>;
