export enum PaseAniTagType {
  title = "title",
  team = "team",
  episode = "episode",
  source_team = "source_team",
  source_type = "source_type",
  resolution = "resolution",
  subtitle_language = "subtitle_language",
  file_type = "file_type",
  video_type = "video_type",
  audio_type = "audio_type",
  link = "link",
  unknown = "unknown",
}

export const TAG_TYPE_TO_COLOR = {
  title: "red",
  team: "orange",
  episode: "yellow",
  source_team: "green",
  source_type: "green",
  resolution: "teal",
  subtitle_language: "blue",
  file_type: "cyan",
  video_type: "purple",
  audio_type: "pink",
  link: "gray",
  unknown: "gray",
};

export interface PaseAniTag {
  type: PaseAniTagType;
  value: string;
  parser: string;
}

export interface PaseAniParseError {
  message: string;
  parser: string;
}

export interface PaseAniResult {
  tags: PaseAniTag[];
  errors: PaseAniParseError[];
}
