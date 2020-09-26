import("./pkg")
.catch(console.error)
.then(module => {
    'use strict';

    // options
    window.plot_options = {
        target: '#plot',
        data: [],
    };
    window.fitness_points = [];
    window.fitness_plot_options = {
        target: '#fitness',
        data: [],
    };
    window.evolution_timeout = null;

    // functions
    window.plot_or_sample = (fn, i) => {
        if (typeof i === 'undefined') i = 0;
        if (i === 0) {
            window.plot_options.data[0] = {
                graphType: 'polyline',
                fn: fn,
            };
        } else {
            window.plot_options.data[1] = {
                fnType: 'points',
                graphType: 'scatter',
                points: window.sample(fn),
            };
        }
        return functionPlot(window.plot_options);
    };
    window.plot_fitness_point = y => {
        window.fitness_points.push(y);
        window.fitness_plot_options.xAxis = { domain: [0, window.fitness_points.length] };
        const lower_y = window.fitness_points.reduce((acc, v) => Math.min(acc, v));
        const upper_y = window.fitness_points.reduce((acc, v) => Math.max(acc, v));
        window.fitness_plot_options.yAxis = { domain: [lower_y-1, upper_y+1] };
        window.fitness_plot_options.data[0] = {
            fnType: 'points',
            graphType: 'polyline',
            points: window.fitness_points.map((y, x) => [x, y]),
        };
        return functionPlot(window.fitness_plot_options);
    };
    window.sample_fn = (fn, beg, end, step) => {
        let samples = [];
        for (let x = beg; x <= end; x += step)
            samples.push([x, fn(x)]);
        return samples;
    };
    window.sample_math_string = (str, beg, end, step) => {
        str = str.replace(/sin/g, 'Math.sin');
        str = str.replace(/cos/g, 'Math.cos');
        str = str.replace(/\^/g, '**');
        const fn = x => {
            let y = eval(str.replace(/x/g, `(${x})`));
            return isNaN(y) ? 0 : y;
        }
        return window.sample_fn(fn, beg, end, step);
    };
    window.sample = fn => {
        const BEG = -5;
        const END = 5;
        const STEP = 0.5;
        if (typeof fn === 'string') return window.sample_math_string(fn, BEG, END, STEP);
        else if (typeof fn === 'function') return window.sample_math_string(fn, BEG, END, STEP);
        else throw `fn must be a function or a string, but ${typeof fn} provided`;
    };
    window.step_evolution = (n) => {
        if (n > 0) {
            window.evolution_timeout = setTimeout(() => {
                // console.log('stepping');
                window.evolution_instance.step(1);
                $('#best_guess').text(window.evolution_instance.best_string());
                window.plot_or_sample(scope => window.evolution_instance.best_eval(scope.x), 0);
                window.plot_fitness_point(window.evolution_instance.best_fitness());

                if (window.evolution_timeout !== null) window.step_evolution(n-1);
            }, 1);
        }
    };
    window.clear_evolution = () => {
        if (window.evolution_timeout !== null) {
            clearTimeout(window.evolution_timeout);
            window.evolution_timeout = null;
        }
    };

    // main
    $(() => {
        window.plot_or_sample((scope) => 0, 0);

        $('#preset_function').on('change', event => {
            let v = $(event.target).val();
            if (v !== '') {
                $('#function_input').val(v).trigger('change');
            }
        });

        let function_input_setup = event => {
            let v = $(event.target).val();
            try {
                window.clear_evolution();
                window.plot_or_sample(v, 1);
            } catch (error) {
                if (!(error instanceof TypeError)) {
                    console.log(error);
                }
            }
        };

        $('#function_input').on('change', function_input_setup);
        $('#function_input').on('keyup', event => {
            $('#preset_function').val('');
            function_input_setup(event);
        });

        $('#go').on('click', () => {
            let f = $('#function_input').val();
            if (f !== '') {
                let samples = window.sample(f);
                let xs = samples.map(e => e[0]);
                let ys = samples.map(e => e[1]);
                window.fitness_points = [];

                window.evolution_instance = module.from_xy(xs, ys);
                window.clear_evolution();
                window.step_evolution(50000);
            }
        });

        $('#stop').on('click', () => {
            window.clear_evolution();
        });
    });
});
