import {
  FiAlertTriangle,
  FiArrowRightCircle,
  FiCloudLightning,
  FiCloudSnow,
  FiGlobe,
  FiLink,
  FiList,
  FiMoreHorizontal,
  FiShuffle,
  FiTarget,
  FiUserCheck,
} from 'react-icons/fi';
import { Feature } from './common/feature';
import { FlowManyChart } from './common/flow-many';
import { Highlights } from './common/highlight';
import { Ribbon } from './common/ribbon';
import { ExamplesView } from './examples-view';

export function IndexPage() {
  return (
    <>
      <div className="w-full overflow-x-hidden">
        <Ribbon>now on alpha</Ribbon>
        <div className="pt-[78px]">
          <h2 className="max-w-screen-[200px] mx-auto px-5 text-[32px] font-extrabold text-[#0f766e] lg:px-0 lg:text-center lg:text-5xl">
            A Fully-Featured, Open-Source
            <br />
            GraphQL Gateway for Your Project
          </h2>
        </div>
      </div>
      <div className="flex flex-col">
        <div className="w-full py-5">
          <div className="container mx-auto box-border flex flex-col px-6">
            <div className="mb-28 flex flex-col items-start lg:items-center lg:text-center">
              <div className="lg:px-8 lg:text-xl">
                <span className="font-medium lg:leading-8">
                  Conductor enriches your GraphQL runtime with powerful features such as caching,{' '}
                  <br className="hidden lg:block" />
                  rate-limiting, federation support and monitoring using a simple configuration
                  file.
                </span>
              </div>
              <div className="mt-8">
                <FlowManyChart />
              </div>
            </div>
            <Feature
              title="Simply Flexible"
              description={
                <div>
                  Conductor can handle many GraphQL sources, combine, transform and federate them.
                  <br className="hidden lg:block" />
                  You can also expose many GraphQL endpoints from a single instance, with different
                  schemas, variations and plugins.
                </div>
              }
            >
              <Highlights
                className="flex-col md:flex-col"
                items={[
                  {
                    title: 'Runs Everywhere',
                    description: (
                      <p className="text-gray-600 dark:text-gray-400">
                        Conductor runs on your own infrastructure. You can run it as a binary or in
                        a containerized environment like Kubernetes or Docker. You can also run
                        Conductor as a CloudFlare Worker (WASM-compiled).
                      </p>
                    ),
                    icon: <FiCloudSnow size={36} />,
                  },
                  {
                    title: 'Federated Execution',
                    description:
                      'We handle all the complexities under the hood to support Apollo Federation including query planning, steps execution, parallelization and response merging. (Fusion support is also on the way!)',
                    icon: <FiShuffle size={36} />,
                  },
                  {
                    title: 'Multiple Endpoints',
                    description:
                      'Expose the same or different GraphQL sources(s) on multiple endpoints: each endpoint can have its own flow and plugins.',
                    icon: <FiList size={36} />,
                  },
                  {
                    title: 'Custom Logic',
                    description:
                      'Easily fork Conductor, and add new custom plugins for your own, that fits your use. Our plugins architecture is easily extended. And you can always pull request your custom plugins if you think can be broadly beneficial.',
                    icon: <FiLink size={36} />,
                  },
                ]}
              />
            </Feature>
            <Feature
              title="Unlimited Extensibility"
              flipped
              className="pb-0"
              description={
                <>
                  <div>
                    <strong>Enterprise features with zero effort at no cost.</strong> Choose from
                    our curated list of plugins, and write custom ones or conditional logic using
                    simple but powerful pre-compiled scripting with VRL (Vector Remap Language).
                  </div>
                  <Highlights
                    className="mt-6 flex-col md:flex-col"
                    items={[
                      {
                        title: 'Response Caching',
                        description:
                          'Connect single or multiple caching stores, and simply plug-in caching to your GraphQL services. We support in-memory, redis and CloudFlare KV stores.',
                        icon: <FiCloudLightning size={36} />,
                      },
                      {
                        title: 'Security',
                        description:
                          'Built-in plugins for popular authentication flows (JWT with JWKS). Also, hardening plugins like rate-limit are available.',
                        icon: <FiUserCheck size={36} />,
                      },
                      {
                        title: 'Monitoring',
                        description:
                          'Monitor your service with built-in support for telemetry (OpenTelemetry, Jaeger, DataDog).',
                        icon: <FiAlertTriangle size={36} />,
                      },
                      // {
                      //   title: 'GraphQL to REST',
                      //   description: (
                      //     <p className="text-gray-600 dark:text-gray-400">
                      //       Expose any GraphQL schemas as REST service, powered by{' '}
                      //       <a href="https://www.the-guild.dev/graphql/sofa-api" target="_blank">
                      //         <strong>SOFA</strong>
                      //       </a>
                      //     </p>
                      //   ),
                      //   icon: <FiArrowRightCircle size={36} />,
                      // },
                      {
                        title: 'and many more',
                        icon: <FiMoreHorizontal size={36} />,
                      },
                    ]}
                  />
                </>
              }
              side={<ExamplesView />}
            />

            <div className="mb-16">
              <h1 className="mb-7 text-[32px] font-extrabold lg:text-5xl">FAQ</h1>

              <h3 className="mb-2 text-[20px] font-semibold lg:text-3xl">
                Is Conductor a drop-in replacement for Federation support in Apollo Router?
              </h3>
              <p>
                Yes. Conductor is a drop-in replacement for Apollo Router, please open issues on our
                github repository if you face any unexpected behavior.
              </p>
              <h3 className="mb-2 mt-4 text-[20px] font-semibold lg:text-3xl">
                How is support for Enterprise features as in Apollo Router?
              </h3>
              <p>
                Conductor offers all of the Enterprise features you expect from other gateways at no
                cost. We have Authentication, Caching, and Open Telementry plugins that are
                extremely easy to use.
              </p>
              <h3 className="mb-2 mt-4 text-[20px] font-semibold lg:text-3xl">
                Does The Guild offer paid support for Conductor?
              </h3>
              <div>
                Yes. We can usually help as the following:
                <ul className="list-inside list-disc">
                  <li>You can request a shared channel to ask any questions for free.</li>
                  <li>Let us study your architecture and guide you.</li>
                  <li>Fully take on introducing Conductor to your infrastructure.</li>
                </ul>
              </div>
              <h3 className="mb-2 mt-4 text-[20px] font-semibold lg:text-3xl">
                Does Conductor integrate with my existing tooling?
              </h3>
              <p>
                It should be! Our tools are built to be agnostic and vendor-free, you can choose and
                mix between our tools and others tools like Apollo Studio, Uplink or GraphQL Hive
                for example.
              </p>
              <h3 className="mb-2 mt-4 text-[20px] font-semibold lg:text-3xl">
                Can I contribute to Conductor?
              </h3>
              <p>
                Of course! Our roadmap is{' '}
                <a
                  className="text-teal-600"
                  target="_blank"
                  href="https://github.com/the-guild-org/conductor/issues/76"
                >
                  publically published
                </a>
                , you can open new issues for requested features or report bugs, you can tell us if
                we need to reprioritize our roadmap, open pull requests.
              </p>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
