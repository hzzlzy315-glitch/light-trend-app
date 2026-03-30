// All Rust structs now have #[serde(rename_all = "camelCase")]

export interface PlatformDetail {
  platform: string;
  score: number;
  url: string;
}

export interface PlatformStats {
  count: number;
  name: string;
}

export interface ClusteredItem {
  id: string;
  title: string;
  description: string | null;
  url: string;
  score: number;
  platform: string;
  platforms: string[];
  platformDetails: PlatformDetail[];
  mentions: number;
  category: string;
  timestamp: string | null;
  geos: string[] | null;
  normalizedScore: number;
  compositeScore: number;
}

export interface TrendingData {
  items: ClusteredItem[];
  byCategory: Record<string, ClusteredItem[]>;
  platformStats: Record<string, PlatformStats>;
  totalItems: number;
  fetchedAt: string;
  elapsed: number;
}
