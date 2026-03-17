const YT_VIDEO_RE =
  /(?:https?:\/\/)?(?:www\.)?(?:youtube\.com\/watch\?(?:.*&)?v=|youtu\.be\/)([A-Za-z0-9_-]{11})/;

const YT_PLAYLIST_RE =
  /(?:https?:\/\/)?(?:www\.)?youtube\.com\/(?:playlist\?(?:.*&)?list=|watch\?(?:.*&)?list=)([A-Za-z0-9_-]+)/;

const YT_WATCH_WITH_LIST_RE =
  /youtube\.com\/watch\?(?:.*&)?v=([A-Za-z0-9_-]{11})(?:.*&)?list=([A-Za-z0-9_-]+)/;

// Matches YouTube channel URLs:
// - youtube.com/@ChannelName
// - youtube.com/channel/UCxxxxx
// - youtube.com/c/ChannelName
// - youtube.com/user/Username
const YT_CHANNEL_RE =
  /(?:https?:\/\/)?(?:www\.)?youtube\.com\/(?:@[\w.-]+|(?:channel|c|user)\/[\w.-]+)/;

// Matches all yt-dlp supported TikTok URL patterns:
// - Video:      tiktok.com/@user/video/1234
// - Short:      vm.tiktok.com/xxx, vt.tiktok.com/xxx, tiktok.com/t/xxx
// - User:       tiktok.com/@user
// - Sound:      tiktok.com/music/name-1234
// - Effect:     tiktok.com/sticker/name-1234
// - Tag:        tiktok.com/tag/name
// - Collection: tiktok.com/@user/collection/name-1234
// - Live:       tiktok.com/@user/live
// - Douyin:     douyin.com/video/1234
const TIKTOK_VIDEO_RE =
  /(?:https?:\/\/)?(?:www\.)?tiktok\.com\/@[\w.-]+\/video\/\d+/;

const TIKTOK_SHORT_RE =
  /(?:https?:\/\/)?(?:(?:vm|vt)\.tiktok\.com|(?:www\.)?tiktok\.com\/t)\/\w+/;

const TIKTOK_USER_RE =
  /(?:https?:\/\/)?(?:www\.)?tiktok\.com\/@[\w.-]+\/?(?:[#?]|$)/;

const TIKTOK_COLLECTION_RE =
  /(?:https?:\/\/)?(?:www\.)?tiktok\.com\/@[\w.-]+\/collection\/[^/?#]+-\d+/;

const TIKTOK_SOUND_RE =
  /(?:https?:\/\/)?(?:www\.)?tiktok\.com\/music\/[\w.-]+-\d+/;

const TIKTOK_TAG_RE =
  /(?:https?:\/\/)?(?:www\.)?tiktok\.com\/tag\/[^/?#&]+/;

const TIKTOK_LIVE_RE =
  /(?:https?:\/\/)?(?:www\.)?tiktok\.com\/@[\w.-]+\/live/;

const DOUYIN_RE =
  /(?:https?:\/\/)?(?:www\.)?douyin\.com\/video\/\d+/;

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

    if (YT_CHANNEL_RE.test(url)) {
      result.push({ type: "playlist", url });
      continue;
    }

    if (YT_VIDEO_RE.test(url)) {
      result.push({ type: "video", url });
      continue;
    }

    if (TIKTOK_VIDEO_RE.test(url) || TIKTOK_SHORT_RE.test(url) || DOUYIN_RE.test(url)) {
      result.push({ type: "video", url });
      continue;
    }

    if (TIKTOK_COLLECTION_RE.test(url) || TIKTOK_SOUND_RE.test(url) || TIKTOK_TAG_RE.test(url)) {
      result.push({ type: "playlist", url });
      continue;
    }

    if (TIKTOK_LIVE_RE.test(url)) {
      result.push({ type: "video", url });
      continue;
    }

    if (TIKTOK_USER_RE.test(url)) {
      result.push({ type: "playlist", url });
      continue;
    }
  }

  return result;
}

export function isLikelyPlaylist(url: string): boolean {
  return /youtube\.com\/playlist\?/.test(url);
}
