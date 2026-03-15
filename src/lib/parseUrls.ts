const YT_VIDEO_RE =
  /(?:https?:\/\/)?(?:www\.)?(?:youtube\.com\/watch\?(?:.*&)?v=|youtu\.be\/)([A-Za-z0-9_-]{11})/;

const YT_PLAYLIST_RE =
  /(?:https?:\/\/)?(?:www\.)?youtube\.com\/(?:playlist\?(?:.*&)?list=|watch\?(?:.*&)?list=)([A-Za-z0-9_-]+)/;

const YT_WATCH_WITH_LIST_RE =
  /youtube\.com\/watch\?(?:.*&)?v=([A-Za-z0-9_-]{11})(?:.*&)?list=([A-Za-z0-9_-]+)/;

export type ParsedUrl =
  | { type: "video"; url: string }
  | { type: "playlist"; url: string }
  | { type: "video_in_playlist"; videoUrl: string; playlistUrl: string };

export function parseInput(raw: string): ParsedUrl[] {
  // Split by newlines, commas, or spaces
  const parts = raw
    .split(/[\n,\s]+/)
    .map((p) => p.trim())
    .filter(Boolean);

  const result: ParsedUrl[] = [];

  for (const part of parts) {
    // Ensure the URL has a protocol
    const url = part.startsWith("http") ? part : `https://${part}`;

    const watchWithList = YT_WATCH_WITH_LIST_RE.exec(url);
    if (watchWithList) {
      const playlistUrl = `https://www.youtube.com/playlist?list=${watchWithList[2]}`;
      result.push({
        type: "video_in_playlist",
        videoUrl: `https://www.youtube.com/watch?v=${watchWithList[1]}`,
        playlistUrl,
      });
      continue;
    }

    if (YT_PLAYLIST_RE.test(url)) {
      result.push({ type: "playlist", url });
      continue;
    }

    if (YT_VIDEO_RE.test(url)) {
      result.push({ type: "video", url });
      continue;
    }

    // Only accept YouTube URLs
    // Non-YouTube URLs are silently ignored
  }

  return result;
}

export function isLikelyPlaylist(url: string): boolean {
  return /youtube\.com\/playlist\?/.test(url);
}
