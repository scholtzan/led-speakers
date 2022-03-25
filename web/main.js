import './static/style.sass';

import("./pkg").then(module => {
    module.run_app();
});
