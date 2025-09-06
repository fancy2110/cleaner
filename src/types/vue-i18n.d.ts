import 'vue-i18n';

declare module 'vue-i18n' {
    // Define the shape of your messages object
    // eslint-disable-next-line no-unused-vars
    interface DefineLocaleMessage {
        app: {
            title: string;
            loading: string;
            noData: string;
            pleaseSelect: string;
            emptyDirectory: string;
            error: string;
        };
        pathNavigator: {
            selectFolder: string;
            upDirectory: string;
            pleaseSelect: string;
        };
        fileStatistics: {
            title: string;
            files: string;
            folders: string;
            totalSize: string;
            fileTypes: {
                image: string;
                video: string;
                audio: string;
                document: string;
                archive: string;
                code: string;
                executable: string;
                other: string;
            };
            noData: string;
            loading: string;
        };
        fileList: {
            filename: string;
            type: string;
            size: string;
            loading: string;
            empty: string;
            actions: {
                delete: string;
            };
            fileTypes: {
                folder: string;
                image: string;
                video: string;
                audio: string;
                document: string;
                archive: string;
                code: string;
                executable: string;
                pdf: string;
                text: string;
                spreadsheet: string;
                presentation: string;
                compressed: string;
                unknown: string;
                other: string;
            };
        };
        selectionSummary: {
            noSelection: string;
            selected: string;
            totalSize: string;
            typeDistribution: string;
            selectedFiles: string;
            actions: {
                delete: string;
                clear: string;
            };
        };
        dialog: {
            confirm: string;
            cancel: string;
            ok: string;
            delete: {
                title: string;
                message: string;
                messageBulk: string;
            };
        };
        sizes: {
            byte: string;
            kilobyte: string;
            megabyte: string;
            gigabyte: string;
            terabyte: string;
        };
    }
}
