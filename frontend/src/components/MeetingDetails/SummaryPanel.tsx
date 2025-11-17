"use client";

import { Summary, SummaryResponse, Transcript } from '@/types';
import { EditableTitle } from '@/components/EditableTitle';
import { BlockNoteSummaryView, BlockNoteSummaryViewRef } from '@/components/AISummary/BlockNoteSummaryView';
import { EmptyStateSummary } from '@/components/EmptyStateSummary';
import { ModelConfig } from '@/components/ModelSettingsModal';
import { SummaryGeneratorButtonGroup } from './SummaryGeneratorButtonGroup';
import { SummaryUpdaterButtonGroup } from './SummaryUpdaterButtonGroup';
import { MeetingChat } from './MeetingChat';
import Analytics from '@/lib/analytics';
import { RefObject, useState } from 'react';

interface SummaryPanelProps {
  meeting: {
    id: string;
    title: string;
    created_at: string;
  };
  meetingTitle: string;
  onTitleChange: (title: string) => void;
  isEditingTitle: boolean;
  onStartEditTitle: () => void;
  onFinishEditTitle: () => void;
  isTitleDirty: boolean;
  summaryRef: RefObject<BlockNoteSummaryViewRef>;
  isSaving: boolean;
  onSaveAll: () => Promise<void>;
  onCopySummary: () => Promise<void>;
  onOpenFolder: () => Promise<void>;
  aiSummary: Summary | null;
  summaryStatus: 'idle' | 'processing' | 'summarizing' | 'regenerating' | 'completed' | 'error';
  transcripts: Transcript[];
  modelConfig: ModelConfig;
  setModelConfig: (config: ModelConfig | ((prev: ModelConfig) => ModelConfig)) => void;
  onSaveModelConfig: (config?: ModelConfig) => Promise<void>;
  onGenerateSummary: (customPrompt: string) => Promise<void>;
  customPrompt: string;
  summaryResponse: SummaryResponse | null;
  onSaveSummary: (summary: Summary | { markdown?: string; summary_json?: any[] }) => Promise<void>;
  onSummaryChange: (summary: Summary) => void;
  onDirtyChange: (isDirty: boolean) => void;
  summaryError: string | null;
  onRegenerateSummary: () => Promise<void>;
  getSummaryStatusMessage: (status: 'idle' | 'processing' | 'summarizing' | 'regenerating' | 'completed' | 'error') => string;
  availableTemplates: Array<{id: string, name: string, description: string}>;
  selectedTemplate: string;
  onTemplateSelect: (templateId: string, templateName: string) => void;
  isModelConfigLoading?: boolean;
}

export function SummaryPanel({
  meeting,
  meetingTitle,
  onTitleChange,
  isEditingTitle,
  onStartEditTitle,
  onFinishEditTitle,
  isTitleDirty,
  summaryRef,
  isSaving,
  onSaveAll,
  onCopySummary,
  onOpenFolder,
  aiSummary,
  summaryStatus,
  transcripts,
  modelConfig,
  setModelConfig,
  onSaveModelConfig,
  onGenerateSummary,
  customPrompt,
  summaryResponse,
  onSaveSummary,
  onSummaryChange,
  onDirtyChange,
  summaryError,
  onRegenerateSummary,
  getSummaryStatusMessage,
  availableTemplates,
  selectedTemplate,
  onTemplateSelect,
  isModelConfigLoading = false
}: SummaryPanelProps) {
  const isSummaryLoading = summaryStatus === 'processing' || summaryStatus === 'summarizing' || summaryStatus === 'regenerating';

  // Tab state - Chat is default (core business)
  const [activeTab, setActiveTab] = useState<'chat' | 'summary'>('chat');

  return (
    <div className="flex-1 min-w-0 flex flex-col bg-white overflow-hidden">
      {/* Tabs Header */}
      <div className="border-b border-gray-200">
        <div className="flex items-center gap-1 px-4 pt-4">
          {/* Chat Tab - Core Business (First) */}
          <button
            onClick={() => {
              setActiveTab('chat');
              Analytics.trackButtonClick('chat_tab', 'summary_panel');
            }}
            className={`
              px-4 py-2 font-medium text-sm rounded-t-lg transition-colors
              ${activeTab === 'chat'
                ? 'bg-white text-blue-600 border-t border-l border-r border-gray-200'
                : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
              }
            `}
          >
            <span className="flex items-center gap-2">
              💬 Chat
            </span>
          </button>

          {/* Summary Tab (Second) */}
          <button
            onClick={() => {
              setActiveTab('summary');
              Analytics.trackButtonClick('summary_tab', 'summary_panel');
            }}
            className={`
              px-4 py-2 font-medium text-sm rounded-t-lg transition-colors
              ${activeTab === 'summary'
                ? 'bg-white text-blue-600 border-t border-l border-r border-gray-200'
                : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
              }
            `}
          >
            <span className="flex items-center gap-2">
              📝 Summary
            </span>
          </button>
        </div>

        {/* Action Buttons - Only show in Summary tab when summary exists */}
        {activeTab === 'summary' && aiSummary && !isSummaryLoading && (
          <div className="flex items-center justify-center w-full px-4 pb-4 pt-2 gap-2">
            {/* Left-aligned: Summary Generator Button Group */}
            <div className="flex-shrink-0">
              <SummaryGeneratorButtonGroup
                modelConfig={modelConfig}
                setModelConfig={setModelConfig}
                onSaveModelConfig={onSaveModelConfig}
                onGenerateSummary={onGenerateSummary}
                customPrompt={customPrompt}
                summaryStatus={summaryStatus}
                availableTemplates={availableTemplates}
                selectedTemplate={selectedTemplate}
                onTemplateSelect={onTemplateSelect}
                hasTranscripts={transcripts.length > 0}
                isModelConfigLoading={isModelConfigLoading}
              />
            </div>

            {/* Right-aligned: Summary Updater Button Group */}
            <div className="flex-shrink-0">
              <SummaryUpdaterButtonGroup
                isSaving={isSaving}
                isDirty={isTitleDirty || (summaryRef.current?.isDirty || false)}
                onSave={onSaveAll}
                onCopy={onCopySummary}
                onFind={() => {
                  // TODO: Implement find in summary functionality
                  console.log('Find in summary clicked');
                }}
                onOpenFolder={onOpenFolder}
                hasSummary={!!aiSummary}
              />
            </div>
          </div>
        )}
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-hidden">
        {/* Chat Tab Content */}
        {activeTab === 'chat' && (
          <div className="h-full">
            <MeetingChat
              meetingId={meeting.id}
              modelProvider={modelConfig.provider || ''}
              modelName={modelConfig.model || ''}
            />
          </div>
        )}

        {/* Summary Tab Content */}
        {activeTab === 'summary' && (
          <>
            {isSummaryLoading ? (
              <div className="flex flex-col h-full">
                {/* Show button group during generation */}
                <div className="flex items-center justify-center pt-8 pb-4">
                  <SummaryGeneratorButtonGroup
                    modelConfig={modelConfig}
                    setModelConfig={setModelConfig}
                    onSaveModelConfig={onSaveModelConfig}
                    onGenerateSummary={onGenerateSummary}
                    customPrompt={customPrompt}
                    summaryStatus={summaryStatus}
                    availableTemplates={availableTemplates}
                    selectedTemplate={selectedTemplate}
                    onTemplateSelect={onTemplateSelect}
                    hasTranscripts={transcripts.length > 0}
                    isModelConfigLoading={isModelConfigLoading}
                  />
                </div>
                {/* Loading spinner */}
                <div className="flex items-center justify-center flex-1">
                  <div className="text-center">
                    <div className="inline-block animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500 mb-4"></div>
                    <p className="text-gray-600">Generating AI Summary...</p>
                  </div>
                </div>
              </div>
            ) : !aiSummary ? (
              <div className="flex flex-col h-full">
                {/* Centered Summary Generator Button Group when no summary */}
                <div className="flex items-center justify-center pt-8 pb-4">
                  <SummaryGeneratorButtonGroup
                    modelConfig={modelConfig}
                    setModelConfig={setModelConfig}
                    onSaveModelConfig={onSaveModelConfig}
                    onGenerateSummary={onGenerateSummary}
                    customPrompt={customPrompt}
                    summaryStatus={summaryStatus}
                    availableTemplates={availableTemplates}
                    selectedTemplate={selectedTemplate}
                    onTemplateSelect={onTemplateSelect}
                    hasTranscripts={transcripts.length > 0}
                    isModelConfigLoading={isModelConfigLoading}
                  />
                </div>
                {/* Empty state message */}
                <EmptyStateSummary
                  onGenerate={() => onGenerateSummary(customPrompt)}
                  hasModel={modelConfig.provider !== null && modelConfig.model !== null}
                  isGenerating={isSummaryLoading}
                />
              </div>
            ) : transcripts?.length > 0 && (
              <div className="h-full overflow-y-auto">
            {summaryResponse && (
              <div className="fixed bottom-0 left-0 right-0 bg-white shadow-lg p-4 max-h-1/3 overflow-y-auto">
                <h3 className="text-lg font-semibold mb-2">Meeting Summary</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="bg-white p-4 rounded-lg shadow-sm">
                    <h4 className="font-medium mb-1">Key Points</h4>
                    <ul className="list-disc pl-4">
                      {summaryResponse.summary.key_points.blocks.map((block, i) => (
                        <li key={i} className="text-sm">{block.content}</li>
                      ))}
                    </ul>
                  </div>
                  <div className="bg-white p-4 rounded-lg shadow-sm mt-4">
                    <h4 className="font-medium mb-1">Action Items</h4>
                    <ul className="list-disc pl-4">
                      {summaryResponse.summary.action_items.blocks.map((block, i) => (
                        <li key={i} className="text-sm">{block.content}</li>
                      ))}
                    </ul>
                  </div>
                  <div className="bg-white p-4 rounded-lg shadow-sm mt-4">
                    <h4 className="font-medium mb-1">Decisions</h4>
                    <ul className="list-disc pl-4">
                      {summaryResponse.summary.decisions.blocks.map((block, i) => (
                        <li key={i} className="text-sm">{block.content}</li>
                      ))}
                    </ul>
                  </div>
                  <div className="bg-white p-4 rounded-lg shadow-sm mt-4">
                    <h4 className="font-medium mb-1">Main Topics</h4>
                    <ul className="list-disc pl-4">
                      {summaryResponse.summary.main_topics.blocks.map((block, i) => (
                        <li key={i} className="text-sm">{block.content}</li>
                      ))}
                    </ul>
                  </div>
                </div>
                {summaryResponse.raw_summary ? (
                  <div className="mt-4">
                    <h4 className="font-medium mb-1">Full Summary</h4>
                    <p className="text-sm whitespace-pre-wrap">{summaryResponse.raw_summary}</p>
                  </div>
                ) : null}
              </div>
            )}
            <div className="p-6 w-full">
              <BlockNoteSummaryView
                ref={summaryRef}
                summaryData={aiSummary}
                onSave={onSaveSummary}
                onSummaryChange={onSummaryChange}
                onDirtyChange={onDirtyChange}
                status={summaryStatus}
                error={summaryError}
                onRegenerateSummary={() => {
                  Analytics.trackButtonClick('regenerate_summary', 'meeting_details');
                  onRegenerateSummary();
                }}
                meeting={{
                  id: meeting.id,
                  title: meetingTitle,
                  created_at: meeting.created_at
                }}
              />
            </div>
                {summaryStatus !== 'idle' && (
                  <div className={`mt-4 mx-6 p-4 rounded-lg ${summaryStatus === 'error' ? 'bg-red-100 text-red-700' :
                    summaryStatus === 'completed' ? 'bg-green-100 text-green-700' :
                      'bg-blue-100 text-blue-700'
                    }`}>
                    <p className="text-sm font-medium">{getSummaryStatusMessage(summaryStatus)}</p>
                  </div>
                )}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
