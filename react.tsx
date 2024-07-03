import { useRouter } from 'next/navigation';
import { useCallback, useMemo } from 'react';
import { cx } from 'class-variance-authority';
import { GetConversationsQuery } from '~/graphql/generated/client';

export const ConversationBox = ({
    data,
    selected,
    basePath,
}: {
    data: GetConversationsQuery['getCreator']['conversations']['items'][number];
    selected: boolean;
    basePath: string;
}) => {
    const router = useRouter();

    const handleClick = useCallback(() => {
        if (!selected) {
            router.push(`${basePath}/${data.id}`);
        }
    }, [data.id, basePath, selected]);

    const lastMessage = useMemo(() => {
        const message = data.message;

        return message;
    }, [data.message]);

    const lastMessageText = useMemo(() => {
        if (lastMessage?.attachments) {
            return `Sent ${lastMessage.attachments} images`;
        }

        if (lastMessage?.text) {
            return lastMessage.text;
        }

        return 'Started a Conversation';
    }, [lastMessage]);


    return (
        <div
            onClick={handleClick}
            className={cx(
                `relative flex cursor-pointer  items-center space-x-3 rounded-sm  p-3 transition ease-in-out hover:opacity-80 active:opacity-90`,
                selected
                    ? 'cursor-default border-transparent bg-[#8976FF]/[.1]'
                    : 'cursor-pointer border-border bg-transparent',
            )}
        >
            <div className="flex-0 w-full min-w-0">
                <div className="focus:outline-none">
                    <div className="mb-1 flex items-center justify-between">
                        <p className="text-md truncate font-medium text-gray-900 dark:text-gray-100">
                            {data.name}
                        </p>
                    </div>
                    <p
                        className={cx(
                            `truncate text-sm md:max-w-xs`,
                            lastMessage.isRead
                                ? 'text-gray-500 dark:text-gray-400'
                                : 'font-medium text-black dark:text-white',
                        )}
                    >
                        {lastMessageText}
                    </p>
                </div>
            </div>
        </div>
    );
};