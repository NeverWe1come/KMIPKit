export interface ComplianceAlert {
  severity: string;
  code: string;
  message: string;
  offset: number;
}

export interface TtlvNode {
  tagHex: string;
  tagName: string;
  itemType: string;
  offset: number;
  headerLength: number;
  valueOffset: number;
  valueLength: number;
  paddedLength: number;
  displayValue: string;
  enumName?: string | null;
  redacted: boolean;
  sensitive: boolean;
  children: TtlvNode[];
}

export interface DecodeResult {
  rawHex: string;
  byteLength: number;
  root: TtlvNode;
  alerts: ComplianceAlert[];
}
