import React from 'react';
import Giscus from '@giscus/react';
import { useColorMode } from '@docusaurus/theme-common';
import { GiscusConfig } from '../../../config/giscus-config';

export interface GiscusCommentsProps {
  repo?: `${string}/${string}`;
  repoId?: string;
  category?: string;
  categoryId?: string;
  mapping?: 'pathname' | 'url' | 'title' | 'og:title' | 'specific' | 'number';
  term?: string;
  reactionsEnabled?: boolean;
  emitMetadata?: boolean;
  inputPosition?: 'top' | 'bottom';
  lang?: string;
  loading?: 'lazy' | 'eager';
}

const GiscusComments: React.FC<GiscusCommentsProps> = ({
  repo = GiscusConfig.repo as `${string}/${string}`,
  repoId = GiscusConfig.repoId,
  category = GiscusConfig.category,
  categoryId = GiscusConfig.categoryId,
  mapping = GiscusConfig.mapping as
    | 'pathname'
    | 'url'
    | 'title'
    | 'og:title'
    | 'specific'
    | 'number',
  term,
  reactionsEnabled = GiscusConfig.reactionsEnabled,
  emitMetadata = GiscusConfig.emitMetadata,
  inputPosition = GiscusConfig.inputPosition as 'top' | 'bottom',
  lang = GiscusConfig.lang,
  loading = GiscusConfig.loading as 'lazy' | 'eager'
}) => {
  const { colorMode } = useColorMode();

  // Check if configuration is available
  if (!repo || !repoId || !category || !categoryId) {
    return (
      <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
        <p>💬 Comments are not configured yet.</p>
        <p>
          Configure Giscus settings in <code>src/config/giscus-config.ts</code>{' '}
          to enable comments.
        </p>
        <p>
          <a
            href="https://giscus.app"
            target="_blank"
            rel="noopener noreferrer"
            style={{ color: '#0969da' }}
          >
            Configure Giscus →
          </a>
        </p>
      </div>
    );
  }

  return (
    <Giscus
      id="comments"
      repo={repo}
      repoId={repoId}
      category={category}
      categoryId={categoryId}
      mapping={mapping}
      term={term}
      reactionsEnabled={reactionsEnabled ? '1' : '0'}
      emitMetadata={emitMetadata ? '1' : '0'}
      inputPosition={inputPosition}
      theme={colorMode === 'dark' ? 'dark' : 'light'}
      lang={lang}
      loading={loading}
    />
  );
};

export default GiscusComments;
