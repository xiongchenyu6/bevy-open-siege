#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <extracted-package-dir>" >&2
  exit 2
fi

PACKAGE_DIR="$1"
if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "content rating audit package directory not found: $PACKAGE_DIR" >&2
  exit 1
fi

for file in \
  CONTENT_RATING.md \
  VERSION.ron \
  STORE_PAGE.md \
  PRESSKIT.md \
  PRIVACY.md \
  QA_SIGNOFF.md; do
  if [[ ! -f "$PACKAGE_DIR/$file" ]]; then
    echo "content rating audit missing file: $file" >&2
    exit 1
  fi
done

require_token() {
  local file="$1"
  local token="$2"
  if ! grep -qi -- "$token" "$PACKAGE_DIR/$file"; then
    echo "content rating audit missing token in $file: $token" >&2
    exit 1
  fi
}

require_token CONTENT_RATING.md "fantasy undead"
require_token CONTENT_RATING.md "non-realistic"
require_token CONTENT_RATING.md "no blood"
require_token CONTENT_RATING.md "no gore"
require_token CONTENT_RATING.md "no gambling"
require_token CONTENT_RATING.md "No in-app purchases"
require_token CONTENT_RATING.md "No online multiplayer"
require_token CONTENT_RATING.md "No sexual content"
require_token CONTENT_RATING.md "Data collection: none"
require_token CONTENT_RATING.md "Manual Review Required"
require_token VERSION.ron "Fantasy undead combat with stylized non-realistic visuals."
require_token STORE_PAGE.md "fantasy undead"
require_token STORE_PAGE.md "no blood"
require_token STORE_PAGE.md "no in-app purchases"
require_token STORE_PAGE.md "no telemetry"
require_token PRESSKIT.md "Fantasy non-realistic combat"
require_token PRESSKIT.md "No blood or gore"
require_token PRIVACY.md "does not include telemetry"
require_token PRIVACY.md "does not intentionally collect"
require_token QA_SIGNOFF.md "Content rating review"
require_token QA_SIGNOFF.md "CONTENT_RATING.md"

echo "content rating audit ok"
echo "document: CONTENT_RATING.md"
echo "metadata note: VERSION.ron content_rating_note"
echo "checked gameplay content: fantasy non-realistic undead combat"
echo "checked visual content: no blood, gore, dismemberment, or realistic injury detail"
echo "checked monetization: no in-app purchases, ads, subscriptions, gambling, loot boxes, or paid random rewards"
echo "checked online/data: no accounts, multiplayer, chat, UGC, telemetry, analytics, cloud saves, or leaderboards"
echo "checked sensitive content: no sexual content, explicit language, substances, self-harm, hate content, or real-world claims"
echo "manual rating review still required: platform questionnaire and regional storefront rules"
