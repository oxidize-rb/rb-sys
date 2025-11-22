import React, { useCallback, useMemo, useState } from "react";
import clsx from "clsx";
import ExecutionEnvironment from "@docusaurus/ExecutionEnvironment";
import { useDoc } from "@docusaurus/plugin-content-docs/client";
import styles from "./styles.module.css";

type CopyStatus = "idle" | "loading" | "success" | "error";

const GITHUB_HOSTNAMES = new Set(["github.com", "www.github.com"]);

const toRawGitHubUrl = (editUrl?: string): string | undefined => {
  if (!editUrl) {
    return undefined;
  }

  try {
    const url = new URL(editUrl);
    if (!GITHUB_HOSTNAMES.has(url.hostname)) {
      return undefined;
    }

    const pathSegments = url.pathname.split("/").filter(Boolean);
    if (pathSegments.length < 4) {
      return undefined;
    }

    const [owner, repo, githubMode, ...rest] = pathSegments;
    if (githubMode !== "tree" && githubMode !== "edit" && githubMode !== "blob") {
      return undefined;
    }

    const branch = rest[0];
    const filePath = rest.slice(1).join("/");
    if (!branch || !filePath) {
      return undefined;
    }

    return `https://raw.githubusercontent.com/${owner}/${repo}/${branch}/${filePath}`;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.warn("[CopyMarkdownButton] Failed to parse editUrl", error);
    return undefined;
  }
};

const copyToClipboard = async (text: string) => {
  if (ExecutionEnvironment.canUseDOM && navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  if (!ExecutionEnvironment.canUseDOM) {
    throw new Error("Clipboard is not available during SSR");
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "");
  textarea.style.position = "absolute";
  textarea.style.left = "-9999px";
  document.body.append(textarea);

  const selection = document.getSelection();
  const selected = selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;

  textarea.select();
  try {
    document.execCommand("copy");
  } finally {
    textarea.remove();
    if (selected) {
      selection?.removeAllRanges();
      selection?.addRange(selected);
    }
  }
};

const statusLabel: Record<CopyStatus, string> = {
  idle: "Copy as Markdown",
  loading: "Copying…",
  success: "Copied!",
  error: "Copy failed",
};

const statusAriaLabel: Record<CopyStatus, string> = {
  idle: "Copy this page as Markdown",
  loading: "Copying this page as Markdown",
  success: "Copied this page as Markdown",
  error: "Copying this page as Markdown failed",
};

type CopyMarkdownButtonProps = {
  className?: string;
};

export default function CopyMarkdownButton({ className }: CopyMarkdownButtonProps) {
  const { metadata } = useDoc();
  const rawUrl = useMemo(() => toRawGitHubUrl(metadata.editUrl), [metadata]);
  const [status, setStatus] = useState<CopyStatus>("idle");

  const handleClick = useCallback(async () => {
    if (!rawUrl) {
      setStatus("error");
      return;
    }

    setStatus("loading");
    try {
      const response = await fetch(rawUrl);
      if (!response.ok) {
        throw new Error(`Failed to load Markdown (${response.status})`);
      }
      const markdown = await response.text();
      await copyToClipboard(markdown);
      setStatus("success");
      window.setTimeout(() => setStatus("idle"), 2000);
    } catch (error) {
      // eslint-disable-next-line no-console
      console.error("[CopyMarkdownButton] Failed to copy markdown", error);
      setStatus("error");
      window.setTimeout(() => setStatus("idle"), 4000);
    }
  }, [rawUrl]);

  if (!rawUrl) {
    return null;
  }

  return (
    <div className={clsx(styles.container, className)}>
      <button
        type="button"
        className={clsx("button", "button--sm", styles.ghostButton, status === "error" && styles.ghostButtonDanger)}
        onClick={handleClick}
        disabled={status === "loading"}
        aria-live="polite"
        aria-label={statusAriaLabel[status]}
      >
        {statusLabel[status]}
      </button>
      <span className={styles.status} role="status" aria-live="polite">
        {status === "error" && "Unable to copy. Please try again."}
      </span>
    </div>
  );
}
